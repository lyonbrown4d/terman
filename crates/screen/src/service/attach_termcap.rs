use std::io::{self, Write};

use super::{
    control_termcap::write_screen_termcap,
    ipc_client::request_endpoint_response,
};
use crate::ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse};

pub(super) fn print_attach_dumptermcap(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    match request_endpoint_response(endpoint, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            session_name,
            cols,
            rows,
            ..
        } => {
            let path = write_screen_termcap(&session_name, cols.unwrap_or(80), rows.unwrap_or(24))?;
            let mut stdout = io::stdout();
            stdout.write_all(b"\r\n")?;
            stdout.write_all(
                terman_common::builtin_screen_control_dumptermcap_complete_hint(
                    &path.display().to_string(),
                )
                .as_bytes(),
            )?;
            stdout.write_all(b"\r\n")?;
            stdout.flush()
        }
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_screen_unexpected_response_hint(&format!("{response:?}")),
        )),
    }
}