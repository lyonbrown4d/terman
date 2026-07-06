use std::{env, io};

use crate::{
    ScreenArgs,
    ipc::{ScreenIpcRequest, ScreenIpcResponse},
};

type SessionRequester = fn(&ScreenArgs, ScreenIpcRequest) -> io::Result<ScreenIpcResponse>;


pub(super) fn request_dinfo_command(
    args: &ScreenArgs,
    request: SessionRequester,
) -> io::Result<()> {
    match request(args, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            session_name,
            attach_clients,
            cols,
            rows,
            active_window,
            ..
        } => {
            let term = env::var("TERM").unwrap_or_else(|_| String::from("unknown"));
            println!(
                "{}",
                terman_common::builtin_screen_control_dinfo_hint(
                    &session_name,
                    attach_clients,
                    cols,
                    rows,
                    active_window,
                    &term,
                )
            );
            Ok(())
        }
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}
pub(super) fn request_info_command(
    args: &ScreenArgs,
    request: SessionRequester,
) -> io::Result<()> {
    match request(args, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            session_name,
            replay_bytes,
            attach_clients,
            cols,
            rows,
            scrollback_lines,
            ..
        } => {
            println!(
                "{}",
                terman_common::builtin_screen_control_info_hint(
                    &session_name,
                    replay_bytes,
                    attach_clients,
                    cols,
                    rows,
                    scrollback_lines,
                )
            );
            Ok(())
        }
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}
fn unexpected_response_error(response: &ScreenIpcResponse) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        terman_common::builtin_screen_control_unexpected_response_hint(&format!("{response:?}")),
    )
}