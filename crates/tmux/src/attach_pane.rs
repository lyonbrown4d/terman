use crate::pane_layout::PaneDirection;
use std::io;

use crate::{
    attach_window::kill_current_window,
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
};

pub(crate) fn split_current_pane(
    endpoint: &TmuxIpcEndpoint,
    horizontal: bool,
) -> io::Result<()> {
    send_request(
        endpoint,
        TmuxIpcRequest::SplitPane {
            window: None,
            horizontal,
            command: None,
        },
    )
}

pub(crate) fn cycle_current_pane_layout(endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
    send_request(endpoint, TmuxIpcRequest::CyclePaneLayout { window: None })
}

pub(crate) fn toggle_current_pane_zoom(endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
    send_request(
        endpoint,
        TmuxIpcRequest::TogglePaneZoom {
            window: None,
            pane: None,
        },
    )
}
pub(crate) fn select_last_pane(endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
    send_request(endpoint, TmuxIpcRequest::SelectLastPane { window: None })
}

pub(crate) fn select_next_pane(endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
    let panes = query_panes(endpoint)?;
    let position = panes
        .pane_indexes
        .iter()
        .position(|pane| *pane == panes.active_pane)
        .unwrap_or(0);
    let Some(pane) = panes
        .pane_indexes
        .get((position + 1) % panes.pane_indexes.len().max(1))
        .copied()
    else {
        return Ok(());
    };
    send_request(
        endpoint,
        TmuxIpcRequest::SelectPane {
            window: Some(panes.window_index),
            pane: Some(pane),
        },
    )
}

pub(crate) fn select_pane_direction(
    endpoint: &TmuxIpcEndpoint,
    direction: PaneDirection,
) -> io::Result<()> {
    send_request(
        endpoint,
        TmuxIpcRequest::SelectPaneDirection {
            window: None,
            direction,
        },
    )
}

pub(crate) fn resize_current_pane(
    endpoint: &TmuxIpcEndpoint,
    direction: PaneDirection,
) -> io::Result<()> {
    send_request(
        endpoint,
        TmuxIpcRequest::ResizePaneDirection {
            window: None,
            pane: None,
            direction,
            adjustment: 1,
        },
    )
}
pub(crate) fn swap_current_pane(
    endpoint: &TmuxIpcEndpoint,
    forward: bool,
) -> io::Result<()> {
    send_request(
        endpoint,
        TmuxIpcRequest::SwapPane {
            window: None,
            source: None,
            target: None,
            forward,
        },
    )
}

pub(crate) fn kill_current_pane(endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
    let panes = query_panes(endpoint)?;
    if panes.pane_indexes.len() <= 1 {
        return kill_current_window(endpoint);
    }
    send_request(
        endpoint,
        TmuxIpcRequest::KillPane {
            window: Some(panes.window_index),
            pane: Some(panes.active_pane),
        },
    )
}

struct AttachedPanes {
    window_index: u32,
    active_pane: u32,
    pane_indexes: Vec<u32>,
}

fn query_panes(endpoint: &TmuxIpcEndpoint) -> io::Result<AttachedPanes> {
    match request_endpoint_response(endpoint, TmuxIpcRequest::PaneInfo { window: None })? {
        TmuxIpcResponse::Panes {
            window_index,
            active_pane,
            pane_indexes,
            ..
        } => Ok(AttachedPanes {
            window_index,
            active_pane,
            pane_indexes,
        }),
        TmuxIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, reason))
        }
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")),
        )),
    }
}

fn send_request(endpoint: &TmuxIpcEndpoint, request: TmuxIpcRequest) -> io::Result<()> {
    match request_endpoint_response(endpoint, request)? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, reason))
        }
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")),
        )),
    }
}
