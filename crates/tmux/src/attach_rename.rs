use std::io;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    attach_status::{query_status_line, render_status_line},
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
    sessions::{RenameBuiltinTmuxWindow, rename_builtin_tmux_window},
};

pub(crate) fn handle_rename_input(
    endpoint: &TmuxIpcEndpoint,
    key: &KeyEvent,
    input: &mut String,
) -> io::Result<()> {
    match key.code {
        KeyCode::Enter => {
            let name = input.trim().to_string();
            if !name.is_empty() {
                rename_current_window(endpoint, name)?;
            }
            render_current_status(endpoint);
        }
        KeyCode::Esc => render_current_status(endpoint),
        KeyCode::Backspace => {
            input.pop();
            let _ = render_status_line(&format!("tmux rename | {input}"));
        }
        KeyCode::Char(ch) if !key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
            input.push(ch);
            let _ = render_status_line(&format!("tmux rename | {input}"));
        }
        _ => {}
    }
    Ok(())
}

fn rename_current_window(endpoint: &TmuxIpcEndpoint, name: String) -> io::Result<()> {
    let (session_name, active_window) = current_session_and_window(endpoint)?;
    match rename_builtin_tmux_window(&session_name, active_window as usize, &name)
        .map_err(|err| io::Error::new(err.kind(), err.to_string()))?
    {
        RenameBuiltinTmuxWindow::Renamed => send_rename(endpoint, active_window, name),
        RenameBuiltinTmuxWindow::SessionMissing => Err(io::Error::new(
            io::ErrorKind::NotFound,
            terman_common::builtin_tmux_session_not_found_hint(&session_name),
        )),
        RenameBuiltinTmuxWindow::WindowMissing => Err(io::Error::new(
            io::ErrorKind::NotFound,
            terman_common::builtin_tmux_window_not_found_hint(&session_name, active_window as usize),
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

fn send_rename(endpoint: &TmuxIpcEndpoint, index: u32, name: String) -> io::Result<()> {
    match request_endpoint_response(endpoint, TmuxIpcRequest::RenameWindow { index, name })? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::PermissionDenied, reason)),
        _ => Ok(()),
    }
}

fn render_current_status(endpoint: &TmuxIpcEndpoint) {
    if let Ok(status) = query_status_line(endpoint) {
        let _ = render_status_line(&status);
    }
}