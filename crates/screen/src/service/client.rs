use std::{
    fs,
    io::{self, BufRead, BufReader, Write},
};

use super::attach::attach_interactive;
use crate::{
    ScreenArgs,
    ipc::{ScreenAttachMode, ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse},
    sessions::find_builtin_screen_session_for_attach,
};

pub(crate) fn request_screen_attach(args: &ScreenArgs) -> io::Result<()> {
    let (mode, target) = match (&args.resume, &args.multi_attach) {
        (Some(target), None) => (ScreenAttachMode::Resume, target.as_deref()),
        (None, Some(target)) => (ScreenAttachMode::MultiAttach, target.as_deref()),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                terman_common::builtin_screen_attach_target_required_hint(),
            ));
        }
    };

    let session = find_builtin_screen_session_for_attach(target)?;
    let endpoint = session
        .ipc_endpoint
        .as_deref()
        .map(ScreenIpcEndpoint::from_raw_name)
        .unwrap_or_else(|| ScreenIpcEndpoint::for_session(&session.name));
    let mut stream = endpoint.connect_options()?.connect_sync()?;
    let request = ScreenIpcRequest::Attach {
        mode,
        target: Some(session.name),
        detach_existing: args.detach_existing,
    };

    serde_json::to_writer(&mut stream, &request)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    attach_interactive(endpoint, stream)
}

pub(crate) fn request_screen_server_ready(session_name: &str) -> io::Result<()> {
    let endpoint = ScreenIpcEndpoint::for_session(session_name);
    send_control_request(&endpoint, ScreenIpcRequest::Ping)
}

pub(crate) fn request_screen_control_command(args: &ScreenArgs) -> io::Result<()> {
    let Some(command_text) = args
        .execute
        .as_deref()
        .map(str::trim)
        .filter(|command| !command.is_empty())
    else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_command_required_hint(),
        ));
    };

    let (command, inline_payload) = split_control_command(command_text);
    match command.to_ascii_lowercase().as_str() {
        "quit" => send_session_control_request(args, ScreenIpcRequest::Quit),
        "detach" => send_session_control_request(args, ScreenIpcRequest::DetachAll),
        "info" => request_session_info(args),
        "hardcopy" => {
            let path = control_command_payload(inline_payload, &args.execute_args);
            if path.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    terman_common::builtin_screen_control_hardcopy_path_required_hint(),
                ));
            }
            request_session_hardcopy(args, &path)
        }
        "resize" => {
            let payload = control_command_payload(inline_payload, &args.execute_args);
            let (cols, rows) = parse_resize_payload(&payload)?;
            send_session_control_request(args, ScreenIpcRequest::Resize { cols, rows })
        }
        "stuff" => {
            let payload = control_command_payload(inline_payload, &args.execute_args);
            if payload.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    terman_common::builtin_screen_control_stuff_required_hint(),
                ));
            }
            send_session_control_request(
                args,
                ScreenIpcRequest::Input {
                    bytes: decode_stuff_payload(&payload),
                },
            )
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_command_unsupported_hint(command),
        )),
    }
}

fn request_session_info(args: &ScreenArgs) -> io::Result<()> {
    match request_session_response(args, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            replay_bytes,
            attach_clients,
        } => {
            println!(
                "{}",
                terman_common::builtin_screen_control_info_hint(replay_bytes, attach_clients)
            );
            Ok(())
        }
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected screen info response",
        )),
    }
}

fn request_session_hardcopy(args: &ScreenArgs, path: &str) -> io::Result<()> {
    match request_session_response(args, ScreenIpcRequest::Hardcopy)? {
        ScreenIpcResponse::Hardcopy { bytes } => {
            fs::write(path, &bytes)?;
            println!(
                "{}",
                terman_common::builtin_screen_control_hardcopy_complete_hint(path, bytes.len())
            );
            Ok(())
        }
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected screen hardcopy response",
        )),
    }
}

fn parse_resize_payload(payload: &str) -> io::Result<(u16, u16)> {
    let mut parts = payload.split_whitespace();
    let Some(cols) = parts.next().and_then(|value| value.parse::<u16>().ok()) else {
        return Err(invalid_resize_payload());
    };
    let Some(rows) = parts.next().and_then(|value| value.parse::<u16>().ok()) else {
        return Err(invalid_resize_payload());
    };
    if cols == 0 || rows == 0 || parts.next().is_some() {
        return Err(invalid_resize_payload());
    }
    Ok((cols, rows))
}

fn invalid_resize_payload() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_resize_required_hint(),
    )
}

fn split_control_command(command: &str) -> (&str, &str) {
    let command = command.trim();
    let command_end = command.find(char::is_whitespace).unwrap_or(command.len());
    let verb = &command[..command_end];
    let payload = command[command_end..].trim_start();
    (verb, payload)
}

fn control_command_payload(inline_payload: &str, args: &[String]) -> String {
    let mut payload = String::new();
    if !inline_payload.is_empty() {
        payload.push_str(inline_payload);
    }
    for arg in args {
        if !payload.is_empty() {
            payload.push(' ');
        }
        payload.push_str(arg);
    }
    payload
}

fn decode_stuff_payload(payload: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(payload.len());
    let mut chars = payload.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            push_utf8(&mut bytes, ch);
            continue;
        }

        match chars.next() {
            Some('n') => bytes.push(b'\n'),
            Some('r') => bytes.push(b'\r'),
            Some('t') => bytes.push(b'\t'),
            Some('\\') => bytes.push(b'\\'),
            Some(other) => {
                bytes.push(b'\\');
                push_utf8(&mut bytes, other);
            }
            None => bytes.push(b'\\'),
        }
    }

    bytes
}

fn push_utf8(bytes: &mut Vec<u8>, ch: char) {
    let mut buf = [0u8; 4];
    bytes.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
}

fn send_session_control_request(args: &ScreenArgs, request: ScreenIpcRequest) -> io::Result<()> {
    match request_session_response(args, request)? {
        ScreenIpcResponse::Accepted => Ok(()),
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected screen control response",
        )),
    }
}

fn request_session_response(
    args: &ScreenArgs,
    request: ScreenIpcRequest,
) -> io::Result<ScreenIpcResponse> {
    let session = find_builtin_screen_session_for_attach(args.session_name.as_deref())?;
    let endpoint = session
        .ipc_endpoint
        .as_deref()
        .map(ScreenIpcEndpoint::from_raw_name)
        .unwrap_or_else(|| ScreenIpcEndpoint::for_session(&session.name));
    request_endpoint_response(&endpoint, request)
}

pub(super) fn send_control_request(
    endpoint: &ScreenIpcEndpoint,
    request: ScreenIpcRequest,
) -> io::Result<()> {
    match request_endpoint_response(endpoint, request)? {
        ScreenIpcResponse::Accepted => Ok(()),
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected screen control response",
        )),
    }
}

fn request_endpoint_response(
    endpoint: &ScreenIpcEndpoint,
    request: ScreenIpcRequest,
) -> io::Result<ScreenIpcResponse> {
    let mut stream = endpoint.connect_options()?.connect_sync()?;
    serde_json::to_writer(&mut stream, &request)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut response = String::new();
    BufReader::new(stream).read_line(&mut response)?;
    serde_json::from_str(response.trim_end())
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}
