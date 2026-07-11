use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    attach_pane::kill_current_pane,
    attach_status::{query_status_line, render_status_line},
    attach_window::kill_current_window,
    ipc::TmuxIpcEndpoint,
};

#[derive(Clone, Copy)]
pub(crate) enum KillTarget {
    Pane,
    Window,
}

pub(crate) fn handle_kill_confirmation(
    endpoint: &TmuxIpcEndpoint,
    key: &KeyEvent,
    target: KillTarget,
) -> io::Result<bool> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            match target {
                KillTarget::Pane => kill_current_pane(endpoint)?,
                KillTarget::Window => kill_current_window(endpoint)?,
            }
            render_current_status(endpoint);
            Ok(false)
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            render_current_status(endpoint);
            Ok(false)
        }
        _ => Ok(true),
    }
}

fn render_current_status(endpoint: &TmuxIpcEndpoint) {
    if let Ok(status) = query_status_line(endpoint) {
        let _ = render_status_line(&status);
    }
}