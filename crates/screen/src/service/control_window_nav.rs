use std::io;

use crate::{
    ScreenArgs,
    ipc::{ScreenIpcRequest, ScreenIpcResponse},
};

type SessionRequester = fn(&ScreenArgs, ScreenIpcRequest) -> io::Result<ScreenIpcResponse>;

pub(super) fn request_window_navigation_command(
    args: &ScreenArgs,
    command: &str,
    request: SessionRequester,
) -> io::Result<()> {
    let request_kind = match command {
        "next" => ScreenIpcRequest::NextWindow,
        "prev" | "previous" => ScreenIpcRequest::PreviousWindow,
        "other" => ScreenIpcRequest::LastWindow,
        _ => return Ok(()),
    };
    match request(args, request_kind)? {
        ScreenIpcResponse::Accepted => Ok(()),
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_screen_control_unexpected_response_hint(&format!(
                "{response:?}"
            )),
        )),
    }
}