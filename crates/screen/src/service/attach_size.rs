use std::io;

use super::ipc_client::{request_endpoint_response, send_control_request};
use crate::ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse};

pub(super) fn fit_attach_window(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    let (cols, rows) = terman_common::current_terminal_size()?;
    send_control_request(endpoint, ScreenIpcRequest::Resize { cols, rows })
}
pub(super) fn toggle_attach_width(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    match request_endpoint_response(endpoint, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info { cols, rows, .. } => send_control_request(
            endpoint,
            ScreenIpcRequest::Resize {
                cols: toggled_width(cols.unwrap_or(80)),
                rows: rows.unwrap_or(24),
            },
        ),
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_screen_unexpected_response_hint(&format!("{response:?}")),
        )),
    }
}

fn toggled_width(cols: u16) -> u16 {
    if cols == 80 { 132 } else { 80 }
}

#[cfg(test)]
mod tests {
    use super::toggled_width;

    #[test]
    fn toggles_attach_width() {
        assert_eq!(toggled_width(80), 132);
        assert_eq!(toggled_width(132), 80);
    }
}