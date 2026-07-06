use std::io;

use crate::{
    ScreenArgs,
    ipc::{ScreenIpcRequest, ScreenIpcResponse},
};

type SessionRequester = fn(&ScreenArgs, ScreenIpcRequest) -> io::Result<ScreenIpcResponse>;

pub(super) fn request_windows_command(
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
            window_title,
            ..
        } => {
            let title = window_title.as_deref().unwrap_or(&session_name);
            println!(
                "{}",
                terman_common::builtin_screen_control_windows_entry_hint(
                    title,
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
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_screen_control_unexpected_response_hint(&format!(
                "{response:?}"
            )),
        )),
    }
}