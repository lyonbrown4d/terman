use std::{io, sync::mpsc};

use interprocess::local_socket::prelude::*;

use crate::{
    pane_service::{accept_control, write_window_missing},
    session_core::{TmuxControlEvent, TmuxSessionBus},
};

pub(crate) fn cycle_pane_layout(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    control_tx: &mpsc::Sender<TmuxControlEvent>,
    window: Option<u32>,
) -> io::Result<()> {
    let Some(status) = bus.pane_status_snapshot(window) else {
        return write_window_missing(
            stream,
            window.unwrap_or_default(),
        );
    };
    accept_control(
        stream,
        control_tx,
        TmuxControlEvent::CyclePaneLayout {
            window: status.window_index,
        },
    )
}