use std::{fs, io};

use super::{
    control_parse::{control_command_payload, decode_stuff_payload, parse_resize_payload},
    control_target::request_with_window_target,
    ipc_client::request_endpoint_response,
};
use crate::{
    ScreenArgs,
    ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse},
    sessions::find_builtin_screen_session_for_attach,
};

pub(super) fn request_echo_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let message = control_command_payload(inline_payload, extra_args);
    if message.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_echo_required_hint(),
        ));
    }
    send_session_control_request(args, ScreenIpcRequest::Echo { message })
}

pub(super) fn request_kill_command(args: &ScreenArgs) -> io::Result<()> {
    send_targeted_session_control_request(args, ScreenIpcRequest::KillWindow)
}
pub(super) fn request_logfile_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    if let Some(seconds) = logfile_flush_seconds(&payload)? {
        return send_targeted_session_control_request(args, ScreenIpcRequest::SetLogFlush { seconds });
    }
    if payload.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_logfile_required_hint(),
        ));
    }
    send_targeted_session_control_request(args, ScreenIpcRequest::SetLogFile { path: payload })
}

fn logfile_flush_seconds(payload: &str) -> io::Result<Option<u64>> {
    let mut parts = payload.split_whitespace();
    let Some(command) = parts.next() else {
        return Ok(None);
    };
    if !command.eq_ignore_ascii_case("flush") {
        return Ok(None);
    }
    let Some(seconds) = parts.next().and_then(|value| value.parse::<u64>().ok()) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_logfile_required_hint(),
        ));
    };
    if parts.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_logfile_required_hint(),
        ));
    }
    Ok(Some(seconds))
}

pub(super) fn request_new_window_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let command = control_command_payload(inline_payload, extra_args);
    let command = command.trim().to_string();
    let command = if command.is_empty() { None } else { Some(command) };
    send_session_control_request(args, ScreenIpcRequest::NewWindow { command })
}
pub(super) fn request_paste_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let register = payload.trim();
    if register.is_empty() {
        send_targeted_session_control_request(args, ScreenIpcRequest::PasteBuffer)
    } else {
        send_targeted_session_control_request(
            args,
            ScreenIpcRequest::PasteRegister {
                name: register.to_string(),
            },
        )
    }
}

pub(super) fn request_pastefile_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let path = control_command_payload(inline_payload, extra_args);
    if path.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_pastefile_path_required_hint(),
        ));
    }
    request_session_pastefile(args, &path)
}

pub(super) fn request_resize_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let (cols, rows) = parse_resize_payload(&payload)?;
    send_session_control_request(args, ScreenIpcRequest::Resize { cols, rows })
}

pub(super) fn request_title_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let title = control_command_payload(inline_payload, extra_args);
    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_title_required_hint(),
        ));
    }
    send_targeted_session_control_request(args, ScreenIpcRequest::SetWindowTitle { title })
}

pub(super) fn request_stuff_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    if payload.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_stuff_required_hint(),
        ));
    }
    send_targeted_session_control_request(
        args,
        ScreenIpcRequest::Input {
            bytes: decode_stuff_payload(&payload),
        },
    )
}

pub(super) fn send_session_control_request(
    args: &ScreenArgs,
    request: ScreenIpcRequest,
) -> io::Result<()> {
    match request_session_response(args, request)? {
        ScreenIpcResponse::Accepted => Ok(()),
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

pub(super) fn send_targeted_session_control_request(
    args: &ScreenArgs,
    request: ScreenIpcRequest,
) -> io::Result<()> {
    match request_with_window_target(args, request, request_session_response)? {
        ScreenIpcResponse::Accepted => Ok(()),
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}
pub(super) fn request_session_response(
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

fn request_session_pastefile(args: &ScreenArgs, path: &str) -> io::Result<()> {
    let bytes = fs::read(path)?;
    send_targeted_session_control_request(args, ScreenIpcRequest::Input { bytes })
}

fn unexpected_response_error(response: &ScreenIpcResponse) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        terman_common::builtin_screen_control_unexpected_response_hint(&format!("{response:?}")),
    )
}