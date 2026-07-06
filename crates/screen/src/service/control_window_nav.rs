use std::io;

use crate::{
    ScreenArgs,
    ipc::{ScreenIpcRequest, ScreenIpcResponse},
};

type SessionRequester = fn(&ScreenArgs, ScreenIpcRequest) -> io::Result<ScreenIpcResponse>;

pub(super) fn request_window_navigation_command(
    args: &ScreenArgs,
    request: SessionRequester,
) -> io::Result<()> {
    match request(args, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info { .. } => Ok(()),
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