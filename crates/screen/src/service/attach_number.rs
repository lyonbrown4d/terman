use std::io;

use super::ipc_client::request_endpoint_response;
use crate::ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse};

pub(super) fn print_attach_number(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    match request_endpoint_response(endpoint, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            active_window,
            windows,
            ..
        } => {
            let title = windows
                .iter()
                .find(|window| window.index == active_window)
                .map(|window| window.title.as_str())
                .unwrap_or_default();
            println!(
                "{}",
                terman_common::builtin_screen_control_number_hint(active_window, title)
            );
            Ok(())
        }
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