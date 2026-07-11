use std::io;

use crate::{
    attach_keys::TmuxPrefixCommand,
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
    sessions::{
        AddBuiltinTmuxWindow, KillBuiltinTmuxWindow, add_builtin_tmux_window,
        kill_builtin_tmux_window,
    },
};

pub(crate) fn handle_window_command(
    endpoint: &TmuxIpcEndpoint,
    command: TmuxPrefixCommand,
) -> io::Result<()> {
    let index = match command {
        TmuxPrefixCommand::SelectWindow(index) => index,
        TmuxPrefixCommand::CreateWindow => return create_window(endpoint),
        TmuxPrefixCommand::KillWindow => return kill_current_window(endpoint),
        TmuxPrefixCommand::NextWindow => next_window_index(endpoint, true)?,
        TmuxPrefixCommand::PreviousWindow => next_window_index(endpoint, false)?,
        TmuxPrefixCommand::CommandPrompt
        | TmuxPrefixCommand::RenameSession
        | TmuxPrefixCommand::RenameWindow
        | TmuxPrefixCommand::ListWindows
        | TmuxPrefixCommand::LastWindow
        | TmuxPrefixCommand::SplitHorizontal
        | TmuxPrefixCommand::SplitVertical
        | TmuxPrefixCommand::NextPane
        | TmuxPrefixCommand::SelectPane(_)
        | TmuxPrefixCommand::SwapPaneUp
        | TmuxPrefixCommand::SwapPaneDown
        | TmuxPrefixCommand::TogglePaneZoom
        | TmuxPrefixCommand::CopyMode
        | TmuxPrefixCommand::PasteBuffer
        | TmuxPrefixCommand::KillPane
        | TmuxPrefixCommand::Help => return Ok(()),
    };
    select_window(endpoint, index)
}

pub(crate) fn current_active_window(endpoint: &TmuxIpcEndpoint) -> io::Result<u32> {
    current_session_and_window(endpoint).map(|(_, active_window)| active_window)
}

pub(crate) fn select_window(endpoint: &TmuxIpcEndpoint, index: u32) -> io::Result<()> {
    send_request(endpoint, TmuxIpcRequest::SelectWindow { index })
}

pub(crate) fn kill_current_window(endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
    let (session_name, active_window) = current_session_and_window(endpoint)?;
    match kill_builtin_tmux_window(&session_name, Some(active_window))
        .map_err(|err| io::Error::new(err.kind(), err.to_string()))?
    {
        KillBuiltinTmuxWindow::Killed { index, .. } => {
            send_request(endpoint, TmuxIpcRequest::KillWindow { index })
        }
        KillBuiltinTmuxWindow::SessionKilled => send_request(endpoint, TmuxIpcRequest::Quit),
        KillBuiltinTmuxWindow::SessionMissing => Err(io::Error::new(
            io::ErrorKind::NotFound,
            terman_common::builtin_tmux_session_not_found_hint(&session_name),
        )),
        KillBuiltinTmuxWindow::WindowMissing => Err(io::Error::new(
            io::ErrorKind::NotFound,
            terman_common::builtin_tmux_window_not_found_hint(
                &session_name,
                active_window as usize,
            ),
        )),
    }
}

fn current_session_and_window(endpoint: &TmuxIpcEndpoint) -> io::Result<(String, u32)> {
    match request_endpoint_response(endpoint, TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info { session_name, active_window, .. } => Ok((session_name, active_window)),
        TmuxIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::PermissionDenied, reason)),
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")),
        )),
    }
}

fn create_window(endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
    let session_name = current_session_name(endpoint)?;
    match add_builtin_tmux_window(&session_name)
        .map_err(|err| io::Error::new(err.kind(), err.to_string()))?
    {
        AddBuiltinTmuxWindow::Added { index, name, .. } => send_request(
            endpoint,
            TmuxIpcRequest::NewWindow { index, name, command: None },
        ),
        AddBuiltinTmuxWindow::SessionMissing => Err(io::Error::new(
            io::ErrorKind::NotFound,
            terman_common::builtin_tmux_session_not_found_hint(&session_name),
        )),
    }
}

fn current_session_name(endpoint: &TmuxIpcEndpoint) -> io::Result<String> {
    match request_endpoint_response(endpoint, TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info { session_name, .. } => Ok(session_name),
        TmuxIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::PermissionDenied, reason)),
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")),
        )),
    }
}

fn next_window_index(endpoint: &TmuxIpcEndpoint, forward: bool) -> io::Result<u32> {
    match request_endpoint_response(endpoint, TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info { active_window, window_indexes, .. } => neighbor_window_index(
            active_window,
            &window_indexes,
            forward,
        )
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                terman_common::builtin_tmux_window_not_found_hint("current", active_window as usize),
            )
        }),
        TmuxIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::PermissionDenied, reason)),
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")),
        )),
    }
}

fn neighbor_window_index(active_window: u32, indexes: &[u32], forward: bool) -> Option<u32> {
    if indexes.is_empty() {
        return None;
    }
    let position = indexes.iter().position(|index| *index == active_window).unwrap_or(0);
    let next = if forward {
        (position + 1) % indexes.len()
    } else if position == 0 {
        indexes.len() - 1
    } else {
        position - 1
    };
    indexes.get(next).copied()
}

fn send_request(endpoint: &TmuxIpcEndpoint, request: TmuxIpcRequest) -> io::Result<()> {
    match request_endpoint_response(endpoint, request)? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::PermissionDenied, reason)),
        _ => Ok(()),
    }
}
