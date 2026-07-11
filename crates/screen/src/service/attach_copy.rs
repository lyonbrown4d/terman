use std::io;

use super::ipc_client::{request_endpoint_response, send_control_request};
use crate::{
    copy_mode::ScreenCopyMode,
    ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse},
};

pub(super) fn start_attach_copy_mode(
    endpoint: &ScreenIpcEndpoint,
) -> io::Result<ScreenCopyMode> {
    let replay = match request_endpoint_response(
        endpoint,
        ScreenIpcRequest::Hardcopy {
            include_history: true,
        },
    )? {
        ScreenIpcResponse::Hardcopy { bytes } => bytes,
        ScreenIpcResponse::Rejected { reason } => {
            return Err(io::Error::new(io::ErrorKind::Unsupported, reason));
        }
        response => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                terman_common::builtin_screen_control_unexpected_response_hint(&format!(
                    "{response:?}"
                )),
            ));
        }
    };
    let (cols, rows) = terman_common::current_terminal_size()?;
    Ok(ScreenCopyMode::from_replay(&replay, cols, rows))
}

pub(super) fn finish_attach_copy_mode(
    endpoint: &ScreenIpcEndpoint,
    copied: Option<Vec<u8>>,
) -> io::Result<()> {
    if let Some(bytes) = copied {
        send_control_request(endpoint, ScreenIpcRequest::SetPasteBuffer { bytes })?;
    }
    send_control_request(endpoint, ScreenIpcRequest::Redisplay)
}