use std::{io, sync::mpsc};

use interprocess::local_socket::prelude::*;

use crate::{
    ipc::TmuxIpcResponse,
    service_codec::write_response,
    session_core::{TmuxControlEvent, TmuxSessionBus},
};

pub(crate) fn set_synchronize_panes(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    control_tx: &mpsc::Sender<TmuxControlEvent>,
    window: Option<u32>,
    enabled: Option<bool>,
) -> io::Result<()> {
    let Some(status) = bus.pane_status_snapshot(window) else {
        let window = window.unwrap_or_default();
        return write_response(
            stream,
            &TmuxIpcResponse::Rejected {
                reason: terman_common::builtin_tmux_window_not_found_hint(
                    "current",
                    window as usize,
                ),
            },
        );
    };
    control_tx
        .send(TmuxControlEvent::SetSynchronizePanes {
            window: status.window_index,
            enabled,
        })
        .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err.to_string()))?;
    write_response(stream, &TmuxIpcResponse::Accepted)
}