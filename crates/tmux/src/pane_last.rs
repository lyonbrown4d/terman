use std::{io, sync::mpsc};

use interprocess::local_socket::prelude::*;

use crate::{
    ipc::TmuxIpcResponse,
    service_codec::write_response,
    session_core::{TmuxControlEvent, TmuxSessionBus},
};

pub(crate) fn select_last_pane(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    control_tx: &mpsc::Sender<TmuxControlEvent>,
    window: Option<u32>,
) -> io::Result<()> {
    let Some(status) = bus.pane_status_snapshot(window) else {
        return write_response(
            stream,
            &TmuxIpcResponse::Rejected {
                reason: terman_common::builtin_tmux_window_not_found_hint(
                    "current",
                    window.unwrap_or_default() as usize,
                ),
            },
        );
    };
    let Some(pane) = status
        .last_pane
        .filter(|pane| status.pane_indexes.contains(pane))
    else {
        return write_response(stream, &TmuxIpcResponse::Accepted);
    };
    control_tx
        .send(TmuxControlEvent::SelectPane {
            window: status.window_index,
            pane,
        })
        .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err.to_string()))?;
    write_response(stream, &TmuxIpcResponse::Accepted)
}