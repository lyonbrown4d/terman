use std::{
    error::Error,
    io::{self, BufRead, BufReader, Write},
    thread,
};

use crossterm::{
    event::{read, Event},
    terminal::{disable_raw_mode, enable_raw_mode, size},
};
use interprocess::local_socket::prelude::*;

use crate::{
    args::target_session_arg,
    attach_keys::key_event_bytes,
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
    let stream = open_attach_stream(endpoint)?;
    let _raw_mode = RawModeGuard::enable()?;
    sync_terminal_size(endpoint)?;
    let _input_thread = spawn_terminal_event_forwarder(endpoint.clone());
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
            | TmuxIpcResponse::Info { .. }
            | TmuxIpcResponse::Resize { .. } => {}
        }
    }
}

fn open_attach_stream(endpoint: &TmuxIpcEndpoint) -> io::Result<LocalSocketStream> {
    let mut stream = endpoint.connect_options()?.connect_sync()?;
    write_attach_request(&mut stream)?;
    Ok(stream)
}

fn write_attach_request(stream: &mut LocalSocketStream) -> io::Result<()> {
    serde_json::to_writer(&mut *stream, &TmuxIpcRequest::Attach).map_err(json_io_error)?;
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

fn spawn_terminal_event_forwarder(endpoint: TmuxIpcEndpoint) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let _ = forward_terminal_events(endpoint);
    })
}

fn forward_terminal_events(endpoint: TmuxIpcEndpoint) -> io::Result<()> {
    loop {
        match read()? {
            Event::Key(key) => {
                if let Some(bytes) = key_event_bytes(key) {
                    send_request(&endpoint, TmuxIpcRequest::Input { bytes })?;
                }
            }
            Event::Resize(cols, rows) => send_resize(&endpoint, cols, rows)?,
            _ => {}
        }
    }
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