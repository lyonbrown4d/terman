use std::io;

use crate::{
    ScreenArgs,
    ipc::{ScreenIpcRequest, ScreenIpcResponse, ScreenWindowInfo},
};

type SessionRequester = fn(&ScreenArgs, ScreenIpcRequest) -> io::Result<ScreenIpcResponse>;

pub(super) fn request_windows_command(
    args: &ScreenArgs,
    request: SessionRequester,
) -> io::Result<()> {
    match request(args, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            attach_clients,
            cols,
            rows,
            windows,
            ..
        } => {
            for window in windows {
                print_window_entry(&window, attach_clients, cols, rows);
            }
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

fn print_window_entry(
    window: &ScreenWindowInfo,
    attach_clients: usize,
    cols: Option<u16>,
    rows: Option<u16>,
) {
    println!(
        "{}",
        terman_common::builtin_screen_control_windows_entry_hint(
            window.index,
            window.active,
            &window.title,
            window.replay_bytes,
            attach_clients,
            cols,
            rows,
        )
    );
}