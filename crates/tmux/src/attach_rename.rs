use std::io;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    attach_status::{query_status_line, render_status_line},
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
    sessions::{
        RenameBuiltinTmuxSession, RenameBuiltinTmuxWindow, rename_builtin_tmux_session,
        rename_builtin_tmux_window,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RenameTarget {
    Session,
    Window,
}

pub(crate) fn render_rename_prompt(target: RenameTarget, input: &str) -> io::Result<()> {
    let prompt = match target {
        RenameTarget::Session => terman_common::builtin_tmux_rename_session_prompt(input),
        RenameTarget::Window => terman_common::builtin_tmux_rename_window_prompt(input),
    };
    render_status_line(&prompt)
}
pub(crate) fn handle_rename_input(
    endpoint: &TmuxIpcEndpoint,
    key: &KeyEvent,
    target: RenameTarget,
    input: &mut String,
) -> io::Result<()> {
    match key.code {
        KeyCode::Enter => {
            let name = input.trim().to_string();
            if !name.is_empty() {
                match target {
                    RenameTarget::Session => rename_current_session(endpoint, name)?,
                    RenameTarget::Window => rename_current_window(endpoint, name)?,
                }
            }
            render_current_status(endpoint);
        }
        KeyCode::Esc => render_current_status(endpoint),
        KeyCode::Backspace => {
            input.pop();
            let _ = render_rename_prompt(target, input);
        }
        KeyCode::Char(ch) if !key.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => {
            input.push(ch);
            let _ = render_rename_prompt(target, input);
        }
        _ => {}
    }
    Ok(())
}

fn rename_current_session(endpoint: &TmuxIpcEndpoint, name: String) -> io::Result<()> {
    let (session_name, _) = current_session_and_window(endpoint)?;
    match rename_builtin_tmux_session(&session_name, &name)
        .map_err(|err| io::Error::new(err.kind(), err.to_string()))?
    {
        RenameBuiltinTmuxSession::Renamed => send_session_rename(endpoint, name),
        RenameBuiltinTmuxSession::SourceMissing => Err(io::Error::new(
            io::ErrorKind::NotFound,
            terman_common::builtin_tmux_session_not_found_hint(&session_name),
        )),
        RenameBuiltinTmuxSession::DestinationExists => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            terman_common::builtin_tmux_session_exists_hint(&name),
        )),
    }
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

fn send_session_rename(endpoint: &TmuxIpcEndpoint, name: String) -> io::Result<()> {
    match request_endpoint_response(endpoint, TmuxIpcRequest::RenameSession { name })? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, reason))
        }
        _ => Ok(()),
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