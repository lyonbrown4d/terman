use std::sync::Mutex;

use std::{
    error::Error,
    io::{self, BufRead, BufReader, Write},
    process,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use crossterm::{
    event::{Event, read},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use interprocess::local_socket::prelude::*;

use crate::{
    args::target_session_arg,
    attach_copy::{finish_attach_copy_mode, start_attach_copy_mode},
    attach_input::{AttachInputMode, AttachInputResult},
    attach_mouse::{
        AttachMouseState, disable_mouse_capture, enable_mouse_capture, handle_attach_mouse,
    },
    attach_status::{query_status_line, render_status_line_with_override},
    copy_mode::{TmuxCopyMode, TmuxCopyResult},
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
    let copy_active = Arc::new(AtomicBool::new(false));
    let status_override = Arc::new(Mutex::new(None::<String>));
    let _input_thread = spawn_terminal_event_forwarder(
        endpoint.clone(),
        client_id,
        copy_active.clone(),
        status_override.clone(),
    );
    let mut reader = BufReader::new(stream);
    let mut status = query_status_line(endpoint).unwrap_or_else(|_| String::from("tmux"));

    loop {
        let Some(response) = read_attach_response(&mut reader)? else {
            return Ok(());
        };
        let display_output = !copy_active.load(Ordering::Acquire);
        match response {
            TmuxIpcResponse::Attached { replay } => {
                if display_output {
                    write_output(&replay)?;
                    status = query_status_line(endpoint).unwrap_or(status);
                    render_status_line_with_override(&status, &status_override)?;
                }
            }
            TmuxIpcResponse::Output { bytes } => {
                if display_output {
                    let refresh = bytes.starts_with(b"\x1bc");
                    write_output(&bytes)?;
                    if refresh {
                        status = query_status_line(endpoint).unwrap_or(status);
                    }
                    render_status_line_with_override(&status, &status_override)?;
                }
            }
            TmuxIpcResponse::Resize { .. } => {
                if display_output {
                    status = query_status_line(endpoint).unwrap_or(status);
                    render_status_line_with_override(&status, &status_override)?;
                }
            }
            TmuxIpcResponse::Detached | TmuxIpcResponse::Exit { .. } => return Ok(()),
            TmuxIpcResponse::Rejected { reason } => {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    reason,
                )));
            }
            TmuxIpcResponse::Accepted
            | TmuxIpcResponse::Buffer { .. }
            | TmuxIpcResponse::Buffers { .. }
            | TmuxIpcResponse::Captured { .. }
            | TmuxIpcResponse::Info { .. }
            | TmuxIpcResponse::Panes { .. } => {}
        }
    }
}

fn open_attach_stream(
    endpoint: &TmuxIpcEndpoint,
    client_id: &str,
) -> io::Result<LocalSocketStream> {
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

fn write_output(bytes: &[u8]) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(bytes)?;
    stdout.flush()
}

fn sync_terminal_size(endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
    let (cols, rows) = terman_common::current_terminal_size()?;
    send_resize(
        endpoint,
        cols,
        terman_common::terminal_rows_without_status(rows),
    )
}

fn spawn_terminal_event_forwarder(
    endpoint: TmuxIpcEndpoint,
    client_id: String,
    copy_active: Arc<AtomicBool>,
    status_override: Arc<Mutex<Option<String>>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let _ = forward_terminal_events(endpoint, client_id, copy_active, status_override);
    })
}

fn forward_terminal_events(
    endpoint: TmuxIpcEndpoint,
    client_id: String,
    copy_active: Arc<AtomicBool>,
    status_override: Arc<Mutex<Option<String>>>,
) -> io::Result<()> {
    let mut input_mode = AttachInputMode::default();
    let mut mouse_state = AttachMouseState::default();
    let mut copy_mode: Option<TmuxCopyMode> = None;
    loop {
        match read()? {
            Event::Key(key) => {
                if let Some(mode) = copy_mode.as_mut() {
                    match mode.handle_key(key) {
                        TmuxCopyResult::Continue => mode.render()?,
                        TmuxCopyResult::Cancel => {
                            copy_mode = None;
                            leave_copy_mode(&endpoint, &copy_active, None)?;
                        }
                        TmuxCopyResult::Copy(bytes) => {
                            copy_mode = None;
                            leave_copy_mode(&endpoint, &copy_active, Some(bytes))?;
                        }
                    }
                    continue;
                }
                let input_result = input_mode.handle_key(&endpoint, &client_id, key)?;
                if let Ok(mut status) = status_override.lock() {
                    *status = input_mode.status_override();
                }
                match input_result {
                    AttachInputResult::Continue => {}
                    AttachInputResult::Stop => return Ok(()),
                    AttachInputResult::EnterCopyMode => {
                        copy_active.store(true, Ordering::Release);
                        match start_attach_copy_mode(&endpoint) {
                            Ok(mode) => {
                                mode.render()?;
                                copy_mode = Some(mode);
                            }
                            Err(error) => {
                                copy_active.store(false, Ordering::Release);
                                return Err(error);
                            }
                        }
                    }
                }
            }
            Event::Mouse(mouse) => {
                if let Some(mode) = copy_mode.as_mut() {
                    if mode.handle_mouse(mouse) {
                        mode.render()?;
                    }
                } else if !input_mode.blocks_mouse() {
                    handle_attach_mouse(&endpoint, &mut mouse_state, mouse)?;
                }
            }
            Event::Resize(cols, rows) => {
                let rows = terman_common::terminal_rows_without_status(rows);
                send_resize(&endpoint, cols, rows)?;
                if let Some(mode) = copy_mode.as_mut() {
                    mode.resize(cols, rows);
                    mode.render()?;
                }
            }
            _ => {}
        }
    }
}

fn leave_copy_mode(
    endpoint: &TmuxIpcEndpoint,
    copy_active: &AtomicBool,
    copied: Option<Vec<u8>>,
) -> io::Result<()> {
    let result = finish_attach_copy_mode(endpoint, copied);
    copy_active.store(false, Ordering::Release);
    result
}

fn send_resize(endpoint: &TmuxIpcEndpoint, cols: u16, rows: u16) -> io::Result<()> {
    send_request(endpoint, TmuxIpcRequest::Resize { cols, rows })
}

fn send_request(endpoint: &TmuxIpcEndpoint, request: TmuxIpcRequest) -> io::Result<()> {
    match request_endpoint_response(endpoint, request)? {
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
        enable_mouse_capture()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        disable_mouse_capture();
        let _ = disable_raw_mode();
    }
}