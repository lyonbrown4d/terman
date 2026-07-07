use std::{
    error::Error,
    io::{self, BufRead, BufReader, Write},
    process, thread,
    time::{SystemTime, UNIX_EPOCH},
};

use crossterm::{
    cursor::{MoveTo, RestorePosition, SavePosition},
    event::{read, Event, KeyEvent},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType},
};
use interprocess::local_socket::prelude::*;

const PREFIX_STATUS: &str = "tmux prefix | c new  n next  p previous  0-9 select  d detach  C-b send-prefix";

use crate::{
    args::target_session_arg,
    attach_keys::{
        is_detach_key, is_key_press, is_tmux_prefix_key, key_event_bytes, tmux_prefix_bytes,
        tmux_prefix_command, TmuxPrefixCommand,
    },
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
    sessions::{AddBuiltinTmuxWindow, add_builtin_tmux_window, load_builtin_tmux_sessions},
};

pub(crate) fn attach_builtin_tmux_session(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_arg(args)?;
    let endpoint = find_session_endpoint(&target)?;
    stream_attached_session(&endpoint)
}

fn required_target_session_arg(args: &[String]) -> Result<String, Box<dyn Error>> {
    target_session_arg(args).ok_or_else(|| {
        Box::new(io::Error::new(io::ErrorKind::InvalidInput, terman_common::builtin_tmux_target_required_hint())) as Box<dyn Error>
    })
}

fn find_session_endpoint(target: &str) -> Result<TmuxIpcEndpoint, Box<dyn Error>> {
    let Some(session) = load_builtin_tmux_sessions()?.into_iter().find(|session| session.name == target) else {
        return Err(Box::new(io::Error::new(io::ErrorKind::NotFound, terman_common::builtin_tmux_session_not_found_hint(target))));
    };
    Ok(session.ipc_endpoint.as_deref().map(TmuxIpcEndpoint::from_raw_name).unwrap_or_else(|| TmuxIpcEndpoint::for_session(&session.name)))
}

fn stream_attached_session(endpoint: &TmuxIpcEndpoint) -> Result<(), Box<dyn Error>> {
    let client_id = new_attach_client_id();
    let stream = open_attach_stream(endpoint, &client_id)?;
    let _raw_mode = RawModeGuard::enable()?;
    sync_terminal_size(endpoint)?;
    let _input_thread = spawn_terminal_event_forwarder(endpoint.clone(), client_id);
    let mut reader = BufReader::new(stream);
    let mut status = query_status_line(endpoint).unwrap_or_else(|_| String::from("tmux"));

    loop {
        let Some(response) = read_attach_response(&mut reader)? else { return Ok(()); };
        match response {
            TmuxIpcResponse::Attached { replay } => {
                write_output(&replay)?;
                status = query_status_line(endpoint).unwrap_or(status);
                render_status_line(&status)?;
            }
            TmuxIpcResponse::Output { bytes } => {
                let refresh = bytes.starts_with(b"\x1bc");
                write_output(&bytes)?;
                if refresh { status = query_status_line(endpoint).unwrap_or(status); }
                render_status_line(&status)?;
            }
            TmuxIpcResponse::Resize { .. } => {
                status = query_status_line(endpoint).unwrap_or(status);
                render_status_line(&status)?;
            }
            TmuxIpcResponse::Detached | TmuxIpcResponse::Exit { .. } => return Ok(()),
            TmuxIpcResponse::Rejected { reason } => return Err(Box::new(io::Error::new(io::ErrorKind::PermissionDenied, reason))),
            TmuxIpcResponse::Accepted | TmuxIpcResponse::Captured { .. } | TmuxIpcResponse::Info { .. } => {}
        }
    }
}

fn open_attach_stream(endpoint: &TmuxIpcEndpoint, client_id: &str) -> io::Result<LocalSocketStream> {
    let mut stream = endpoint.connect_options()?.connect_sync()?;
    write_attach_request(&mut stream, client_id)?;
    Ok(stream)
}

fn write_attach_request(stream: &mut LocalSocketStream, client_id: &str) -> io::Result<()> {
    serde_json::to_writer(&mut *stream, &TmuxIpcRequest::Attach { client_id: Some(client_id.to_string()) }).map_err(json_io_error)?;
    stream.write_all(b"\n")?;
    stream.flush()
}

fn read_attach_response(reader: &mut BufReader<LocalSocketStream>) -> io::Result<Option<TmuxIpcResponse>> {
    let mut line = String::new();
    let bytes = reader.read_line(&mut line)?;
    if bytes == 0 { return Ok(None); }
    serde_json::from_str(line.trim_end()).map(Some).map_err(json_io_error)
}

fn write_output(bytes: &[u8]) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(bytes)?;
    stdout.flush()
}

fn query_status_line(endpoint: &TmuxIpcEndpoint) -> io::Result<String> {
    match request_endpoint_response(endpoint, TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info { session_name, active_window, window_indexes, window_names, .. } => {
            let windows = window_indexes.iter().enumerate().map(|(position, index)| {
                let name = window_names.get(position).map(String::as_str).unwrap_or("-");
                if *index == active_window { format!("[{index}:{name}]") } else { format!("{index}:{name}") }
            }).collect::<Vec<_>>().join(" ");
            Ok(format!("tmux {session_name} | {windows} | C-b n/p/0-9 switch  C-b d detach"))
        }
        TmuxIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::PermissionDenied, reason)),
        response => Err(io::Error::new(io::ErrorKind::InvalidData, terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")))),
    }
}

fn render_status_line(status: &str) -> io::Result<()> {
    let (cols, rows) = size()?;
    let row = rows.saturating_sub(1);
    let text = fit_status_text(status, cols as usize);
    let mut stdout = io::stdout().lock();
    execute!(
        stdout,
        SavePosition,
        MoveTo(0, row),
        SetBackgroundColor(Color::DarkBlue),
        SetForegroundColor(Color::White),
        Clear(ClearType::CurrentLine),
        Print(text),
        ResetColor,
        RestorePosition
    )?;
    stdout.flush()
}

fn fit_status_text(status: &str, width: usize) -> String {
    let mut text = status.chars().take(width).collect::<String>();
    let len = text.chars().count();
    if len < width { text.push_str(&" ".repeat(width - len)); }
    text
}

fn sync_terminal_size(endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
    let (cols, rows) = size()?;
    send_resize(endpoint, cols, content_rows(rows))
}

fn spawn_terminal_event_forwarder(endpoint: TmuxIpcEndpoint, client_id: String) -> thread::JoinHandle<()> {
    thread::spawn(move || { let _ = forward_terminal_events(endpoint, client_id); })
}

fn forward_terminal_events(endpoint: TmuxIpcEndpoint, client_id: String) -> io::Result<()> {
    let mut input_mode = AttachInputMode::default();
    loop {
        match read()? {
            Event::Key(key) => if !input_mode.handle_key(&endpoint, &client_id, key)? { return Ok(()); },
            Event::Resize(cols, rows) => send_resize(&endpoint, cols, content_rows(rows))?,
            _ => {}
        }
    }
}

#[derive(Default)]
struct AttachInputMode { prefix_pending: bool }

impl AttachInputMode {
    fn handle_key(&mut self, endpoint: &TmuxIpcEndpoint, client_id: &str, key: KeyEvent) -> io::Result<bool> {
        if !is_key_press(&key) { return Ok(true); }
        if self.prefix_pending {
            self.prefix_pending = false;
            if is_detach_key(&key) {
                send_request(endpoint, TmuxIpcRequest::DetachClient { client_id: client_id.to_string() })?;
                return Ok(false);
            }
            if let Some(command) = tmux_prefix_command(&key) {
                handle_prefix_command(endpoint, command)?;
                let _ = render_current_status(endpoint);
                return Ok(true);
            }
            send_input(endpoint, tmux_prefix_bytes())?;
            let _ = render_current_status(endpoint);
            if is_tmux_prefix_key(&key) { return Ok(true); }
        }
        if is_tmux_prefix_key(&key) {
            self.prefix_pending = true;
            let _ = render_status_line(PREFIX_STATUS);
            return Ok(true);
        }
        if let Some(bytes) = key_event_bytes(&key) { send_input(endpoint, bytes)?; }
        Ok(true)
    }
}

fn handle_prefix_command(endpoint: &TmuxIpcEndpoint, command: TmuxPrefixCommand) -> io::Result<()> {
    let index = match command {
        TmuxPrefixCommand::SelectWindow(index) => index,
        TmuxPrefixCommand::CreateWindow => return create_window(endpoint),
        TmuxPrefixCommand::NextWindow => next_window_index(endpoint, true)?,
        TmuxPrefixCommand::PreviousWindow => next_window_index(endpoint, false)?,
    };
    send_request(endpoint, TmuxIpcRequest::SelectWindow { index })
}

fn create_window(endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
    let session_name = current_session_name(endpoint)?;
    match add_builtin_tmux_window(&session_name).map_err(|err| io::Error::new(err.kind(), err.to_string()))? {
        AddBuiltinTmuxWindow::Added { index, name, .. } => {
            send_request(endpoint, TmuxIpcRequest::NewWindow { index, name, command: None })
        }
        AddBuiltinTmuxWindow::SessionMissing => Err(io::Error::new(
            io::ErrorKind::NotFound,
            terman_common::builtin_tmux_session_not_found_hint(&session_name),
        )),
    }
}

fn current_session_name(endpoint: &TmuxIpcEndpoint) -> io::Result<String> {
    match request_endpoint_response(endpoint, TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info { session_name, .. } => Ok(session_name),
        TmuxIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::PermissionDenied, reason)),
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")),
        )),
    }
}
fn next_window_index(endpoint: &TmuxIpcEndpoint, forward: bool) -> io::Result<u32> {
    match request_endpoint_response(endpoint, TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info { active_window, window_indexes, .. } => neighbor_window_index(active_window, &window_indexes, forward).ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, terman_common::builtin_tmux_window_not_found_hint("current", active_window as usize))),
        TmuxIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::PermissionDenied, reason)),
        response => Err(io::Error::new(io::ErrorKind::InvalidData, terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")))),
    }
}

fn neighbor_window_index(active_window: u32, indexes: &[u32], forward: bool) -> Option<u32> {
    if indexes.is_empty() { return None; }
    let position = indexes.iter().position(|index| *index == active_window).unwrap_or(0);
    let next = if forward { (position + 1) % indexes.len() } else if position == 0 { indexes.len() - 1 } else { position - 1 };
    indexes.get(next).copied()
}

fn render_current_status(endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
    let status = query_status_line(endpoint)?;
    render_status_line(&status)
}
fn send_input(endpoint: &TmuxIpcEndpoint, bytes: Vec<u8>) -> io::Result<()> { send_request(endpoint, TmuxIpcRequest::Input { bytes }) }
fn send_resize(endpoint: &TmuxIpcEndpoint, cols: u16, rows: u16) -> io::Result<()> { send_request(endpoint, TmuxIpcRequest::Resize { cols, rows }) }
fn content_rows(rows: u16) -> u16 { rows.saturating_sub(1).max(1) }

fn send_request(endpoint: &TmuxIpcEndpoint, request: TmuxIpcRequest) -> io::Result<()> {
    match request_endpoint_response(endpoint, request)? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::PermissionDenied, reason)),
        _ => Ok(()),
    }
}

fn new_attach_client_id() -> String {
    let entropy = SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_nanos()).unwrap_or_default();
    format!("{:x}-{entropy:x}", process::id())
}

fn json_io_error(error: serde_json::Error) -> io::Error { io::Error::new(io::ErrorKind::InvalidData, error) }

struct RawModeGuard;

impl RawModeGuard {
    fn enable() -> io::Result<Self> { enable_raw_mode()?; Ok(Self) }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) { let _ = disable_raw_mode(); }
}