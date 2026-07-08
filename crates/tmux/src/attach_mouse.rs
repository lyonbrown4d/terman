use std::io;

use crossterm::{
    event::{MouseButton, MouseEvent, MouseEventKind},
    terminal::size,
};

use crate::{
    attach_keys::TmuxPrefixCommand,
    attach_status::{query_status_line, render_status_line},
    attach_window::{handle_window_command, select_window},
    attach_window_list::{render_window_list_status, TmuxWindowListLayout},
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
    terminal_mouse::mouse_event_bytes,
};

#[derive(Default)]
pub(crate) struct AttachMouseState {
    window_list: Option<TmuxWindowListLayout>,
    suppress_button_release: bool,
}

impl AttachMouseState {
    fn clear(&mut self) { self.window_list = None; }
    fn set_window_list(&mut self, layout: TmuxWindowListLayout) { self.window_list = Some(layout); }
    fn window_at(&self, column: u16) -> Option<u32> { self.window_list.as_ref()?.window_at(column) }
    fn list_open(&self) -> bool { self.window_list.is_some() }
    fn suppress_button_release(&mut self) { self.suppress_button_release = true; }
    fn take_suppressed_button_release(&mut self) -> bool {
        let suppress = self.suppress_button_release;
        self.suppress_button_release = false;
        suppress
    }
}

pub(crate) use terman_common::{disable_mouse_capture, enable_mouse_capture};

pub(crate) fn handle_attach_mouse(
    endpoint: &TmuxIpcEndpoint,
    state: &mut AttachMouseState,
    event: MouseEvent,
) -> io::Result<()> {
    if matches!(event.kind, MouseEventKind::Up(_))
        && state.take_suppressed_button_release()
    {
        return Ok(());
    }
    if !on_status_row(event.row) {
        if state.list_open() {
            if matches!(event.kind, MouseEventKind::Down(_) | MouseEventKind::Drag(_)) {
                state.suppress_button_release();
            }
            state.clear();
            return render_current_status(endpoint);
        }
        state.clear();
        return forward_pane_mouse(endpoint, event);
    }
    match event.kind {
        MouseEventKind::ScrollUp | MouseEventKind::ScrollLeft => select_relative_window(endpoint, state, false),
        MouseEventKind::ScrollDown | MouseEventKind::ScrollRight => select_relative_window(endpoint, state, true),
        MouseEventKind::Down(MouseButton::Left) => { state.suppress_button_release(); select_clicked_window(endpoint, state, event.column) }
        MouseEventKind::Up(MouseButton::Left) => select_clicked_window(endpoint, state, event.column),
        MouseEventKind::Drag(MouseButton::Left) => { state.suppress_button_release(); select_dragged_window(endpoint, state, event.column) }
        MouseEventKind::Down(MouseButton::Right) => show_window_list(endpoint, state),
        MouseEventKind::Down(MouseButton::Middle) => { state.clear(); render_status_line(&terman_common::builtin_tmux_attach_help()) }
        _ => Ok(()),
    }
}

fn show_window_list(endpoint: &TmuxIpcEndpoint, state: &mut AttachMouseState) -> io::Result<()> {
    state.set_window_list(render_window_list_status(endpoint)?);
    Ok(())
}

fn forward_pane_mouse(endpoint: &TmuxIpcEndpoint, event: MouseEvent) -> io::Result<()> {
    let Some(bytes) = mouse_event_bytes(event) else { return Ok(()); };
    match request_endpoint_response(endpoint, TmuxIpcRequest::Input { bytes })? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::PermissionDenied, reason)),
        response => Err(io::Error::new(io::ErrorKind::InvalidData, terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")))),
    }
}

fn select_relative_window(endpoint: &TmuxIpcEndpoint, state: &mut AttachMouseState, forward: bool) -> io::Result<()> {
    state.clear();
    let command = if forward { TmuxPrefixCommand::NextWindow } else { TmuxPrefixCommand::PreviousWindow };
    handle_window_command(endpoint, command)?;
    render_current_status(endpoint)
}

fn select_clicked_window(endpoint: &TmuxIpcEndpoint, state: &mut AttachMouseState, column: u16) -> io::Result<()> {
    let list_open = state.list_open();
    if let Some(index) = state.window_at(column) {
        state.clear();
        select_window(endpoint, index)?;
        return render_current_status(endpoint);
    }
    if list_open {
        state.clear();
        return render_current_status(endpoint);
    }
    match clicked_status_target(endpoint, column)? {
        Some(StatusClickTarget::Window(index)) => { select_window(endpoint, index)?; render_current_status(endpoint)?; }
        Some(StatusClickTarget::Help) => render_status_line(&terman_common::builtin_tmux_attach_help())?,
        None => {}
    }
    Ok(())
}

fn select_dragged_window(endpoint: &TmuxIpcEndpoint, state: &mut AttachMouseState, column: u16) -> io::Result<()> {
    let list_open = state.list_open();
    if let Some(index) = state.window_at(column) {
        state.clear();
        select_window(endpoint, index)?;
        return render_current_status(endpoint);
    }
    if list_open {
        state.clear();
        return render_current_status(endpoint);
    }
    if let Some(StatusClickTarget::Window(index)) = clicked_status_target(endpoint, column)? {
        select_window(endpoint, index)?;
        render_current_status(endpoint)?;
    }
    Ok(())
}
fn clicked_status_target(endpoint: &TmuxIpcEndpoint, column: u16) -> io::Result<Option<StatusClickTarget>> {
    let max_width = size().ok().map(|(cols, _)| cols);
    match request_endpoint_response(endpoint, TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info { session_name, active_window, window_indexes, window_names, .. } => {
            if let Some(index) = status_window_at(column, &session_name, active_window, &window_indexes, &window_names, max_width) {
                Ok(Some(StatusClickTarget::Window(index)))
            } else if status_help_at(column, &session_name, active_window, &window_indexes, &window_names, max_width) {
                Ok(Some(StatusClickTarget::Help))
            } else {
                Ok(None)
            }
        }
        TmuxIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::PermissionDenied, reason)),
        response => Err(io::Error::new(io::ErrorKind::InvalidData, terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")))),
    }
}

enum StatusClickTarget { Window(u32), Help }

fn status_window_at(
    column: u16,
    session_name: &str,
    active_window: u32,
    indexes: &[u32],
    names: &[String],
    max_width: Option<u16>,
) -> Option<u32> {
    if max_width.is_some_and(|max| column >= max) {
        return None;
    }
    let mut offset = terman_common::terminal_text_width(&format!("tmux {session_name} | "));
    for (position, index) in indexes.iter().enumerate() {
        let name = names.get(position).map(String::as_str).unwrap_or("-");
        let label = if *index == active_window { format!("[{index}:{name}]") } else { format!("{index}:{name}") };
        let end = clipped_end(offset, terman_common::terminal_text_width(label.as_str()), max_width);
        if column >= offset && column < end { return Some(*index); }
        offset = offset.saturating_add(terman_common::terminal_text_width(label.as_str()) + 1);
    }
    None
}

fn status_help_at(column: u16, session_name: &str, active_window: u32, indexes: &[u32], names: &[String], max_width: Option<u16>) -> bool {
    let start = status_prompt_start(session_name, active_window, indexes, names);
    column >= start && max_width.map(|max| start < max && column < max).unwrap_or(true)
}

fn status_prompt_start(session_name: &str, active_window: u32, indexes: &[u32], names: &[String]) -> u16 {
    let mut width = terman_common::terminal_text_width(&format!("tmux {session_name} | "));
    for (position, index) in indexes.iter().enumerate() {
        let name = names.get(position).map(String::as_str).unwrap_or("-");
        let label = if *index == active_window { format!("[{index}:{name}]") } else { format!("{index}:{name}") };
        width = width.saturating_add(terman_common::terminal_text_width(label.as_str()));
        if position + 1 < indexes.len() { width = width.saturating_add(1); }
    }
    width.saturating_add(3)
}

fn clipped_end(start: u16, width: u16, max_width: Option<u16>) -> u16 {
    let end = start.saturating_add(width);
    max_width.map(|max| end.min(max)).unwrap_or(end)
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
        assert_eq!(status_window_at(11, "dev", 0, &indexes, &names, None), Some(0));
        assert_eq!(status_window_at(19, "dev", 0, &indexes, &names, None), Some(1));
    }

    #[test]
    fn maps_unicode_window_status_columns() {
        let indexes = vec![0, 1];
        let names = vec![String::from("服务"), String::from("api")];
        assert_eq!(status_window_at(11, "dev", 0, &indexes, &names, None), Some(0));
        assert_eq!(status_window_at(20, "dev", 0, &indexes, &names, None), Some(1));
    }

    #[test]
    fn clamps_window_status_columns_to_visible_width() {
        let indexes = vec![0, 1];
        let names = vec![String::from("服务服务服务"), String::from("api")];
        assert_eq!(status_window_at(18, "dev", 0, &indexes, &names, Some(18)), None);
        assert_eq!(status_window_at(17, "dev", 0, &indexes, &names, Some(18)), Some(0));
    }
}
