use std::{io, sync::mpsc};

use interprocess::local_socket::prelude::*;

use crate::{
    ipc::TmuxIpcResponse,
    service_codec::write_response,
    session_core::{TmuxControlEvent, TmuxPaneStatus, TmuxSessionBus},
};

pub(crate) fn capture_pane(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    window: Option<u32>,
    pane: Option<u32>,
) -> io::Result<()> {
    let Some(status) = bus.pane_status_snapshot(window) else {
        return write_window_missing(stream, window.unwrap_or_default());
    };
    let pane = pane.unwrap_or(status.active_pane);
    match bus.pane_capture_snapshot(Some(status.window_index), Some(pane)) {
        Some(bytes) => write_response(stream, &TmuxIpcResponse::Captured { bytes }),
        None => write_pane_missing(stream, status.window_index, pane),
    }
}

pub(crate) fn clear_history(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    window: Option<u32>,
    pane: Option<u32>,
) -> io::Result<()> {
    let Some(status) = bus.pane_status_snapshot(window) else {
        return write_window_missing(stream, window.unwrap_or_default());
    };
    let pane = pane.unwrap_or(status.active_pane);
    if bus.clear_pane_capture(Some(status.window_index), Some(pane)) {
        write_response(stream, &TmuxIpcResponse::Accepted)
    } else {
        write_pane_missing(stream, status.window_index, pane)
    }
}

pub(crate) fn write_pane_info(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    window: Option<u32>,
) -> io::Result<()> {
    let Some(status) = bus.pane_status_snapshot(window) else {
        return write_window_missing(stream, window.unwrap_or_default());
    };
    write_response(
        stream,
        &TmuxIpcResponse::Panes {
            window_index: status.window_index,
            window_name: status.window_name,
            active_pane: status.active_pane,
            pane_indexes: status.pane_indexes,
        },
    )
}

pub(crate) fn split_pane(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    control_tx: &mpsc::Sender<TmuxControlEvent>,
    window: Option<u32>,
    horizontal: bool,
    command: Option<String>,
) -> io::Result<()> {
    let Some(status) = bus.pane_status_snapshot(window) else {
        return write_window_missing(stream, window.unwrap_or_default());
    };
    accept_control(
        stream,
        control_tx,
        TmuxControlEvent::SplitPane {
            window: status.window_index,
            horizontal,
            command,
        },
    )
}

pub(crate) fn select_pane(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    control_tx: &mpsc::Sender<TmuxControlEvent>,
    window: Option<u32>,
    pane: Option<u32>,
) -> io::Result<()> {
    let Some((status, pane)) = resolve_pane(stream, bus, window, pane)? else {
        return Ok(());
    };
    accept_control(
        stream,
        control_tx,
        TmuxControlEvent::SelectPane {
            window: status.window_index,
            pane,
        },
    )
}

pub(crate) fn swap_pane(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    control_tx: &mpsc::Sender<TmuxControlEvent>,
    window: Option<u32>,
    source: Option<u32>,
    target: Option<u32>,
    forward: bool,
) -> io::Result<()> {
    let Some((status, source)) = resolve_pane(stream, bus, window, source)? else {
        return Ok(());
    };
    let target = target.unwrap_or_else(|| adjacent_pane(&status.pane_indexes, source, forward));
    if !status.pane_indexes.contains(&target) {
        return write_pane_missing(stream, status.window_index, target);
    }
    accept_control(
        stream,
        control_tx,
        TmuxControlEvent::SwapPane {
            window: status.window_index,
            source,
            target,
        },
    )
}
pub(crate) fn kill_pane(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    control_tx: &mpsc::Sender<TmuxControlEvent>,
    window: Option<u32>,
    pane: Option<u32>,
) -> io::Result<()> {
    let Some((status, pane)) = resolve_pane(stream, bus, window, pane)? else {
        return Ok(());
    };
    accept_control(
        stream,
        control_tx,
        TmuxControlEvent::KillPane {
            window: status.window_index,
            pane,
        },
    )
}

pub(crate) fn toggle_pane_zoom(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    control_tx: &mpsc::Sender<TmuxControlEvent>,
    window: Option<u32>,
    pane: Option<u32>,
) -> io::Result<()> {
    let Some((status, pane)) = resolve_pane(stream, bus, window, pane)? else {
        return Ok(());
    };
    accept_control(
        stream,
        control_tx,
        TmuxControlEvent::TogglePaneZoom {
            window: status.window_index,
            pane,
        },
    )
}
pub(crate) fn resize_pane(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    control_tx: &mpsc::Sender<TmuxControlEvent>,
    target: (Option<u32>, Option<u32>),
    size: (Option<u16>, Option<u16>),
) -> io::Result<()> {
    let (window, pane) = target;
    let Some((status, pane)) = resolve_pane(stream, bus, window, pane)? else {
        return Ok(());
    };
    accept_control(
        stream,
        control_tx,
        TmuxControlEvent::ResizePane {
            window: status.window_index,
            pane,
            cols: size.0,
            rows: size.1,
        },
    )
}

fn adjacent_pane(panes: &[u32], pane: u32, forward: bool) -> u32 {
    let position = panes.iter().position(|candidate| *candidate == pane).unwrap_or(0);
    let offset = if forward { 1 } else { panes.len().saturating_sub(1) };
    panes
        .get((position + offset) % panes.len().max(1))
        .copied()
        .unwrap_or(pane)
}
fn resolve_pane(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    window: Option<u32>,
    pane: Option<u32>,
) -> io::Result<Option<(TmuxPaneStatus, u32)>> {
    let Some(status) = bus.pane_status_snapshot(window) else {
        write_window_missing(stream, window.unwrap_or_default())?;
        return Ok(None);
    };
    let pane = pane.unwrap_or(status.active_pane);
    if !status.pane_indexes.contains(&pane) {
        write_pane_missing(stream, status.window_index, pane)?;
        return Ok(None);
    }
    Ok(Some((status, pane)))
}

fn accept_control(
    stream: &mut LocalSocketStream,
    control_tx: &mpsc::Sender<TmuxControlEvent>,
    event: TmuxControlEvent,
) -> io::Result<()> {
    control_tx
        .send(event)
        .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err.to_string()))?;
    write_response(stream, &TmuxIpcResponse::Accepted)
}

fn write_window_missing(stream: &mut LocalSocketStream, window: u32) -> io::Result<()> {
    write_response(
        stream,
        &TmuxIpcResponse::Rejected {
            reason: terman_common::builtin_tmux_window_not_found_hint(
                "current",
                window as usize,
            ),
        },
    )
}

fn write_pane_missing(
    stream: &mut LocalSocketStream,
    window: u32,
    pane: u32,
) -> io::Result<()> {
    write_response(
        stream,
        &TmuxIpcResponse::Rejected {
            reason: terman_common::builtin_tmux_pane_not_found_hint("current", window, pane),
        },
    )
}

pub(crate) fn select_pane_direction(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    control_tx: &mpsc::Sender<TmuxControlEvent>,
    window: Option<u32>,
    direction: crate::pane_layout::PaneDirection,
) -> io::Result<()> {
    let Some(status) = bus.pane_status_snapshot(window) else {
        return write_window_missing(stream, window.unwrap_or_default());
    };
    accept_control(
        stream,
        control_tx,
        TmuxControlEvent::SelectPaneDirection {
            window: status.window_index,
            direction,
        },
    )
}
