use std::{fs, io};

use super::{
    control_parse::{control_command_payload, decode_stuff_payload, parse_resize_payload},
    ipc_client::request_endpoint_response,
};
use crate::{
    ScreenArgs,
    ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse},
    sessions::find_builtin_screen_session_for_attach,
};

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
        "quit" | "kill" => send_session_control_request(args, ScreenIpcRequest::Quit),
        "detach" => send_session_control_request(args, ScreenIpcRequest::DetachAll),
        "clear" => send_session_control_request(args, ScreenIpcRequest::Clear),
        "reset" => send_session_control_request(args, ScreenIpcRequest::Reset),
        "info" => request_session_info(args),
        "hardcopy" => request_hardcopy_command(args, inline_payload),
        "pastefile" => request_pastefile_command(args, inline_payload),
        "resize" => request_resize_command(args, inline_payload),
        "stuff" => request_stuff_command(args, inline_payload),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_command_unsupported_hint(command),
        )),
    }
}

fn request_hardcopy_command(args: &ScreenArgs, inline_payload: &str) -> io::Result<()> {
    let path = control_command_payload(inline_payload, &args.execute_args);
    if path.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_hardcopy_path_required_hint(),
        ));
    }
    request_session_hardcopy(args, &path)
}

fn request_pastefile_command(args: &ScreenArgs, inline_payload: &str) -> io::Result<()> {
    let path = control_command_payload(inline_payload, &args.execute_args);
    if path.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_pastefile_path_required_hint(),
        ));
    }
    request_session_pastefile(args, &path)
}

fn request_resize_command(args: &ScreenArgs, inline_payload: &str) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, &args.execute_args);
    let (cols, rows) = parse_resize_payload(&payload)?;
    send_session_control_request(args, ScreenIpcRequest::Resize { cols, rows })
}

fn request_stuff_command(args: &ScreenArgs, inline_payload: &str) -> io::Result<()> {
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

fn request_session_info(args: &ScreenArgs) -> io::Result<()> {
    match request_session_response(args, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            session_name,
            replay_bytes,
            attach_clients,
            cols,
            rows,
        } => {
            println!(
                "{}",
                terman_common::builtin_screen_control_info_hint(
                    &session_name,
                    replay_bytes,
                    attach_clients,
                    cols,
                    rows,
                )
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

fn request_session_pastefile(args: &ScreenArgs, path: &str) -> io::Result<()> {
    let bytes = fs::read(path)?;
    send_session_control_request(args, ScreenIpcRequest::Input { bytes })
}

fn split_control_command(command: &str) -> (&str, &str) {
    let command = command.trim();
    let command_end = command.find(char::is_whitespace).unwrap_or(command.len());
    let verb = &command[..command_end];
    let payload = command[command_end..].trim_start();
    (verb, payload)
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