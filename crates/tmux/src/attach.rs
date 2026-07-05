use std::{
    error::Error,
    io::{self, BufRead, BufReader, Write},
    process, thread,
    time::{SystemTime, UNIX_EPOCH},
};

use crossterm::{
    event::{read, Event, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, size},
};
use interprocess::local_socket::prelude::*;

use crate::{
    args::target_session_arg,
    attach_keys::{
        is_detach_key, is_key_press, is_tmux_prefix_key, key_event_bytes, tmux_prefix_bytes,
    },
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
    sessions::load_builtin_tmux_sessions,
};

pub(crate) fn attach_builtin_tmux_session(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_arg(args)?;
    let endpoint = find_session_endpoint(&target)?;

    stream_attached_session(&endpoint)
}

fn required_target_session_arg(args: &[String]) -> Result<String, Box<dyn Error>> {
    target_session_arg(args).ok_or_else(|| {
        Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_tmux_target_required_hint(),
        )) as Box<dyn Error>
    })
}

fn find_session_endpoint(target: &str) -> Result<TmuxIpcEndpoint, Box<dyn Error>> {
    let Some(session) = load_builtin_tmux_sessions()?
        .into_iter()
        .find(|session| session.name == target)
    else {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            terman_common::builtin_tmux_session_not_found_hint(target),
        )));
    };

    Ok(session
        .ipc_endpoint
        .as_deref()
        .map(TmuxIpcEndpoint::from_raw_name)
        .unwrap_or_else(|| TmuxIpcEndpoint::for_session(&session.name)))
}

fn stream_attached_session(endpoint: &TmuxIpcEndpoint) -> Result<(), Box<dyn Error>> {
    let client_id = new_attach_client_id();
    let stream = open_attach_stream(endpoint, &client_id)?;
    let _raw_mode = RawModeGuard::enable()?;
    sync_terminal_size(endpoint)?;
    let _input_thread = spawn_terminal_event_forwarder(endpoint.clone(), client_id);
    let mut reader = BufReader::new(stream);
    let mut stdout = io::stdout().lock();

    loop {
        let Some(response) = read_attach_response(&mut reader)? else {
            return Ok(());
        };

        match response {
            TmuxIpcResponse::Attached { replay } => write_output(&mut stdout, &replay)?,
            TmuxIpcResponse::Output { bytes } => write_output(&mut stdout, &bytes)?,
            TmuxIpcResponse::Detached => return Ok(()),
            TmuxIpcResponse::Exit { .. } => return Ok(()),
            TmuxIpcResponse::Rejected { reason } => {
                return Err(Box::new(io::Error::new(io::ErrorKind::PermissionDenied, reason)));
            }
            TmuxIpcResponse::Accepted
            | TmuxIpcResponse::Captured { .. }
            | TmuxIpcResponse::Info { .. }
            | TmuxIpcResponse::Resize { .. } => {}
        }
    }
}

fn open_attach_stream(endpoint: &TmuxIpcEndpoint, client_id: &str) -> io::Result<LocalSocketStream> {
    let mut stream = endpoint.connect_options()?.connect_sync()?;
    write_attach_request(&mut stream, client_id)?;
    Ok(stream)
}

fn write_attach_request(stream: &mut LocalSocketStream, client_id: &str) -> io::Result<()> {
    serde_json::to_writer(
        &mut *stream,
        &TmuxIpcRequest::Attach {
            client_id: Some(client_id.to_string()),
        },
    )
    .map_err(json_io_error)?;
    stream.write_all(b"\n")?;
    stream.flush()
}

fn read_attach_response(
    reader: &mut BufReader<LocalSocketStream>,
) -> io::Result<Option<TmuxIpcResponse>> {
    let mut line = String::new();
    let bytes = reader.read_line(&mut line)?;
    if bytes == 0 {
        return Ok(None);
    }

    serde_json::from_str(line.trim_end())
        .map(Some)
        .map_err(json_io_error)
}

fn write_output(stdout: &mut dyn Write, bytes: &[u8]) -> io::Result<()> {
    stdout.write_all(bytes)?;
    stdout.flush()
}

fn sync_terminal_size(endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
    let (cols, rows) = size()?;
    send_resize(endpoint, cols, rows)
}

fn spawn_terminal_event_forwarder(
    endpoint: TmuxIpcEndpoint,
    client_id: String,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let _ = forward_terminal_events(endpoint, client_id);
    })
}

fn forward_terminal_events(endpoint: TmuxIpcEndpoint, client_id: String) -> io::Result<()> {
    let mut input_mode = AttachInputMode::default();
    loop {
        match read()? {
            Event::Key(key) => {
                if !input_mode.handle_key(&endpoint, &client_id, key)? {
                    return Ok(());
                }
            }
            Event::Resize(cols, rows) => send_resize(&endpoint, cols, rows)?,
            _ => {}
        }
    }
}

#[derive(Default)]
struct AttachInputMode {
    prefix_pending: bool,
}

impl AttachInputMode {
    fn handle_key(
        &mut self,
        endpoint: &TmuxIpcEndpoint,
        client_id: &str,
        key: KeyEvent,
    ) -> io::Result<bool> {
        if !is_key_press(&key) {
            return Ok(true);
        }

        if self.prefix_pending {
            self.prefix_pending = false;
            if is_detach_key(&key) {
                send_request(
                    endpoint,
                    TmuxIpcRequest::DetachClient {
                        client_id: client_id.to_string(),
                    },
                )?;
                return Ok(false);
            }
            send_input(endpoint, tmux_prefix_bytes())?;
            if is_tmux_prefix_key(&key) {
                return Ok(true);
            }
        }

        if is_tmux_prefix_key(&key) {
            self.prefix_pending = true;
            return Ok(true);
        }

        if let Some(bytes) = key_event_bytes(&key) {
            send_input(endpoint, bytes)?;
        }
        Ok(true)
    }
}

fn send_input(endpoint: &TmuxIpcEndpoint, bytes: Vec<u8>) -> io::Result<()> {
    send_request(endpoint, TmuxIpcRequest::Input { bytes })
}

fn send_resize(endpoint: &TmuxIpcEndpoint, cols: u16, rows: u16) -> io::Result<()> {
    send_request(endpoint, TmuxIpcRequest::Resize { cols, rows })
}

fn send_request(endpoint: &TmuxIpcEndpoint, request: TmuxIpcRequest) -> io::Result<()> {
    let response = request_endpoint_response(endpoint, request)?;
    match response {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, reason))
        }
        _ => Ok(()),
    }
}

fn new_attach_client_id() -> String {
    let entropy = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{:x}-{entropy:x}", process::id())
}

fn json_io_error(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

struct RawModeGuard;

impl RawModeGuard {
    fn enable() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}