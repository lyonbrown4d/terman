use std::io;

use super::{
    control_hardcopy_support::{
        HardcopyOptions, hardcopy_options, hardcopydir_path, parse_hardcopy_append,
        unexpected_response_error, write_hardcopy,
    },
    control_parse::control_command_payload,
    control_session::send_session_control_request,
    control_target::request_with_window_target,
};
use crate::{ScreenArgs, ipc::{ScreenIpcRequest, ScreenIpcResponse}};

pub(super) fn request_hardcopy_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let options = hardcopy_options(args, &payload)?;
    request_session_hardcopy(args, &options)
}

pub(super) fn request_hardcopy_append_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let append = parse_hardcopy_append(&payload)?;
    send_session_control_request(args, ScreenIpcRequest::SetHardcopyAppend { append })
}

pub(super) fn request_hardcopydir_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let path = hardcopydir_path(&payload)?;
    send_session_control_request(args, ScreenIpcRequest::SetHardcopyDir { path })
}

fn request_session_hardcopy(args: &ScreenArgs, options: &HardcopyOptions) -> io::Result<()> {
    let request = ScreenIpcRequest::Hardcopy { include_history: options.include_history };
    match request_with_window_target(args, request, super::control_session::request_session_response)? {
        ScreenIpcResponse::Hardcopy { bytes } => {
            write_hardcopy(&options.path, options.append, &bytes)?;
            let path = options.path.display().to_string();
            println!(
                "{}",
                terman_common::builtin_screen_control_hardcopy_complete_hint(&path, bytes.len())
            );
            Ok(())
        }
        ScreenIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::Unsupported, reason)),
        response => Err(unexpected_response_error(&response)),
    }
}