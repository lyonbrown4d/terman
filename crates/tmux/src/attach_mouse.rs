use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, MouseButton, MouseEvent, MouseEventKind},
    execute,
    terminal::size,
};

use crate::{
    attach_keys::TmuxPrefixCommand,
    attach_status::{query_status_line, render_status_line},
    attach_window::{handle_window_command, select_window},
    attach_window_list::render_window_list_status,
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
    terminal_mouse::mouse_event_bytes,
};

pub(crate) fn enable_mouse_capture() -> io::Result<()> {
    execute!(io::stdout(), EnableMouseCapture)
}

pub(crate) fn disable_mouse_capture() {
    let _ = execute!(io::stdout(), DisableMouseCapture);
}

pub(crate) fn handle_attach_mouse(endpoint: &TmuxIpcEndpoint, event: MouseEvent) -> io::Result<()> {
    if !on_status_row(event.row) {
        return forward_pane_mouse(endpoint, event);
    }
    match event.kind {
        MouseEventKind::ScrollUp => select_relative_window(endpoint, false),
        MouseEventKind::ScrollDown => select_relative_window(endpoint, true),
        MouseEventKind::Down(MouseButton::Left) => select_clicked_window(endpoint, event.column),
        MouseEventKind::Down(MouseButton::Right) => render_window_list_status(endpoint),
        MouseEventKind::Down(MouseButton::Middle) => {
            render_status_line(&terman_common::builtin_tmux_attach_help())
        }
        _ => Ok(()),
    }
}

fn forward_pane_mouse(endpoint: &TmuxIpcEndpoint, event: MouseEvent) -> io::Result<()> {
    let Some(bytes) = mouse_event_bytes(event) else {
        return Ok(());
    };
    match request_endpoint_response(endpoint, TmuxIpcRequest::Input { bytes })? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::PermissionDenied, reason)),
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")),
        )),
    }
}
fn select_relative_window(endpoint: &TmuxIpcEndpoint, forward: bool) -> io::Result<()> {
    let command = if forward {
        TmuxPrefixCommand::NextWindow
    } else {
        TmuxPrefixCommand::PreviousWindow
    };
    handle_window_command(endpoint, command)?;
    render_current_status(endpoint)
}

fn select_clicked_window(endpoint: &TmuxIpcEndpoint, column: u16) -> io::Result<()> {
    match clicked_status_target(endpoint, column)? {
        Some(StatusClickTarget::Window(index)) => {
            select_window(endpoint, index)?;
            render_current_status(endpoint)?;
        }
        Some(StatusClickTarget::Help) => render_status_line(&terman_common::builtin_tmux_attach_help())?,
        None => {}
    }
    Ok(())
}

fn clicked_status_target(endpoint: &TmuxIpcEndpoint, column: u16) -> io::Result<Option<StatusClickTarget>> {
    match request_endpoint_response(endpoint, TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info { session_name, active_window, window_indexes, window_names, .. } => {
            if let Some(index) = status_window_at(column, &session_name, active_window, &window_indexes, &window_names) {
                Ok(Some(StatusClickTarget::Window(index)))
            } else if status_help_at(column, &session_name, active_window, &window_indexes, &window_names) {
                Ok(Some(StatusClickTarget::Help))
            } else {
                Ok(None)
            }
        }
        TmuxIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::PermissionDenied, reason)),
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")),
        )),
    }
}

enum StatusClickTarget {
    Window(u32),
    Help,
}

fn status_window_at(
    column: u16,
    session_name: &str,
    active_window: u32,
    indexes: &[u32],
    names: &[String],
) -> Option<u32> {
    let mut offset = format!("tmux {session_name} | ").chars().count() as u16;
    for (position, index) in indexes.iter().enumerate() {
        let name = names.get(position).map(String::as_str).unwrap_or("-");
        let label = if *index == active_window {
            format!("[{index}:{name}]")
        } else {
            format!("{index}:{name}")
        };
        let width = label.chars().count() as u16;
        if column >= offset && column < offset.saturating_add(width) {
            return Some(*index);
        }
        offset = offset.saturating_add(width + 1);
    }
    None
}

fn status_help_at(
    column: u16,
    session_name: &str,
    active_window: u32,
    indexes: &[u32],
    names: &[String],
) -> bool {
    let prompt_start = status_prompt_start(session_name, active_window, indexes, names);
    column >= prompt_start
}

fn status_prompt_start(
    session_name: &str,
    active_window: u32,
    indexes: &[u32],
    names: &[String],
) -> u16 {
    let mut width = format!("tmux {session_name} | ").chars().count() as u16;
    for (position, index) in indexes.iter().enumerate() {
        let name = names.get(position).map(String::as_str).unwrap_or("-");
        let label = if *index == active_window {
            format!("[{index}:{name}]")
        } else {
            format!("{index}:{name}")
        };
        width = width.saturating_add(label.chars().count() as u16);
        if position + 1 < indexes.len() {
            width = width.saturating_add(1);
        }
    }
    width.saturating_add(3)
}
fn on_status_row(row: u16) -> bool {
    size().map(|(_, rows)| row == rows.saturating_sub(1)).unwrap_or(false)
}

fn render_current_status(endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
    let status = query_status_line(endpoint)?;
    render_status_line(&status)
}

#[cfg(test)]
mod tests {
    use super::status_window_at;

    #[test]
    fn maps_window_status_columns() {
        let indexes = vec![0, 1];
        let names = vec![String::from("zsh"), String::from("api")];
        assert_eq!(status_window_at(11, "dev", 0, &indexes, &names), Some(0));
        assert_eq!(status_window_at(19, "dev", 0, &indexes, &names), Some(1));
    }
}