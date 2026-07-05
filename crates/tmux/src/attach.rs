use std::{
    error::Error,
    io::{self, BufRead, BufReader, Write},
    thread,
};

use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use interprocess::local_socket::prelude::*;

use crate::{
    args::target_session_arg,
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
    let _input_thread = spawn_input_forwarder(endpoint.clone());
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

fn spawn_input_forwarder(endpoint: TmuxIpcEndpoint) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let _ = forward_terminal_input(endpoint);
    })
}

fn forward_terminal_input(endpoint: TmuxIpcEndpoint) -> io::Result<()> {
    loop {
        let Event::Key(key) = read()? else {
            continue;
        };
        let Some(bytes) = key_event_bytes(key) else {
            continue;
        };

        send_input(&endpoint, bytes)?;
    }
}

fn send_input(endpoint: &TmuxIpcEndpoint, bytes: Vec<u8>) -> io::Result<()> {
    let response = request_endpoint_response(endpoint, TmuxIpcRequest::Input { bytes })?;
    match response {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, reason))
        }
        _ => Ok(()),
    }
}

fn key_event_bytes(key: KeyEvent) -> Option<Vec<u8>> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    match key.code {
        KeyCode::Backspace => Some(vec![0x7f]),
        KeyCode::Enter => Some(vec![b'\r']),
        KeyCode::Left => Some(ansi_bytes("\x1b[D")),
        KeyCode::Right => Some(ansi_bytes("\x1b[C")),
        KeyCode::Up => Some(ansi_bytes("\x1b[A")),
        KeyCode::Down => Some(ansi_bytes("\x1b[B")),
        KeyCode::Home => Some(ansi_bytes("\x1b[H")),
        KeyCode::End => Some(ansi_bytes("\x1b[F")),
        KeyCode::PageUp => Some(ansi_bytes("\x1b[5~")),
        KeyCode::PageDown => Some(ansi_bytes("\x1b[6~")),
        KeyCode::Tab => Some(vec![b'\t']),
        KeyCode::BackTab => Some(ansi_bytes("\x1b[Z")),
        KeyCode::Delete => Some(ansi_bytes("\x1b[3~")),
        KeyCode::Insert => Some(ansi_bytes("\x1b[2~")),
        KeyCode::Esc => Some(vec![0x1b]),
        KeyCode::Char(ch) => char_key_bytes(ch, key.modifiers),
        KeyCode::F(number) => function_key_bytes(number),
        _ => None,
    }
}

fn char_key_bytes(ch: char, modifiers: KeyModifiers) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    if modifiers.contains(KeyModifiers::ALT) {
        bytes.push(0x1b);
    }

    if modifiers.contains(KeyModifiers::CONTROL) {
        bytes.extend(control_char_bytes(ch)?);
    } else {
        bytes.extend(encoded_char_bytes(ch));
    }

    Some(bytes)
}

fn control_char_bytes(ch: char) -> Option<Vec<u8>> {
    let upper = ch.to_ascii_uppercase();
    if upper.is_ascii_uppercase() {
        return Some(vec![(upper as u8) - b'A' + 1]);
    }

    match ch {
        ' ' => Some(vec![0x00]),
        '[' => Some(vec![0x1b]),
        '\\' => Some(vec![0x1c]),
        ']' => Some(vec![0x1d]),
        '^' => Some(vec![0x1e]),
        '_' => Some(vec![0x1f]),
        '?' => Some(vec![0x7f]),
        _ => None,
    }
}

fn function_key_bytes(number: u8) -> Option<Vec<u8>> {
    match number {
        1 => Some(ansi_bytes("\x1bOP")),
        2 => Some(ansi_bytes("\x1bOQ")),
        3 => Some(ansi_bytes("\x1bOR")),
        4 => Some(ansi_bytes("\x1bOS")),
        5 => Some(ansi_bytes("\x1b[15~")),
        6 => Some(ansi_bytes("\x1b[17~")),
        7 => Some(ansi_bytes("\x1b[18~")),
        8 => Some(ansi_bytes("\x1b[19~")),
        9 => Some(ansi_bytes("\x1b[20~")),
        10 => Some(ansi_bytes("\x1b[21~")),
        11 => Some(ansi_bytes("\x1b[23~")),
        12 => Some(ansi_bytes("\x1b[24~")),
        _ => None,
    }
}

fn encoded_char_bytes(ch: char) -> Vec<u8> {
    let mut buffer = [0; 4];
    ch.encode_utf8(&mut buffer).as_bytes().to_vec()
}

fn ansi_bytes(sequence: &str) -> Vec<u8> {
    sequence.as_bytes().to_vec()
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