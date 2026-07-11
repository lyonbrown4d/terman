use std::io::{self, Write};

use crossterm::event::{KeyEvent, MouseButton, MouseEvent, MouseEventKind};

use super::{
    attach_output::print_attach_help,
    ipc_client::{request_endpoint_response, send_control_request},
};
use crate::{
    ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse},
    mouse_window_list::MouseWindowListState,
    terminal_mouse::mouse_event_bytes,
    window_list_input::{WindowListKeyAction, handle_window_list_key},
};

pub(super) type AttachMouseState = MouseWindowListState;

pub(super) use terman_common::{disable_mouse_capture, enable_mouse_capture};

pub(super) fn open_attach_window_list(
    endpoint: &ScreenIpcEndpoint,
    state: &mut AttachMouseState,
) -> io::Result<()> {
    show_window_list(endpoint, state, current_control_row())
}

pub(super) fn handle_attach_window_list_key(
    endpoint: &ScreenIpcEndpoint,
    state: &mut AttachMouseState,
    key: &KeyEvent,
) -> io::Result<bool> {
    if !state.list_open() {
        return Ok(false);
    }
    match handle_window_list_key(state, key) {
        WindowListKeyAction::Redraw => open_attach_window_list(endpoint, state)?,
        WindowListKeyAction::Select(index) => {
            state.clear();
            send_control_request(endpoint, ScreenIpcRequest::SelectWindow { index })?;
        }
        WindowListKeyAction::Cancel => {
            state.clear();
            send_control_request(endpoint, ScreenIpcRequest::Redisplay)?;
        }
        WindowListKeyAction::Noop => {}
    }
    Ok(true)
}
pub(super) fn handle_attach_mouse(
    endpoint: &ScreenIpcEndpoint,
    state: &mut AttachMouseState,
    event: MouseEvent,
) -> io::Result<()> {
    if matches!(event.kind, MouseEventKind::Up(_)) && state.take_suppressed_button_release() {
        return Ok(());
    }
    if state.list_open()
        && matches!(event.kind, MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left))
    {
        return Ok(());
    }
    if state.list_open() && matches!(event.kind, MouseEventKind::Up(MouseButton::Left)) {
        return select_list_window(endpoint, state, event.row, event.column);
    }
    if state.list_open() {
        if matches!(event.kind, MouseEventKind::Down(_) | MouseEventKind::Drag(_)) {
            state.suppress_button_release();
        }
        state.clear();
        return send_control_request(endpoint, ScreenIpcRequest::Redisplay);
    }

    let control_row = on_control_row(event.row);
    match event.kind {
        MouseEventKind::ScrollUp | MouseEventKind::ScrollLeft if control_row => {
            state.clear();
            select_previous_window(endpoint)
        }
        MouseEventKind::ScrollDown | MouseEventKind::ScrollRight if control_row => {
            state.clear();
            select_next_window(endpoint)
        }
        MouseEventKind::Down(MouseButton::Right) if control_row => show_window_list(endpoint, state, event.row),
        MouseEventKind::Down(MouseButton::Middle) if control_row => {
            state.clear();
            print_attach_help()
        }
        MouseEventKind::Up(MouseButton::Right) | MouseEventKind::Up(MouseButton::Middle)
            if control_row => Ok(()),
        _ => {
            state.clear();
            forward_mouse_event(endpoint, event)
        }
    }
}

fn show_window_list(
    endpoint: &ScreenIpcEndpoint,
    state: &mut AttachMouseState,
    anchor_row: u16,
) -> io::Result<()> {
    match request_endpoint_response(endpoint, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info { attach_clients, cols, rows, windows, .. } => {
            let active = windows.iter().find(|window| window.active)
                .map(|window| window.index).unwrap_or_default();
            state.sync_windows(windows.iter().map(|window| window.index).collect(), active);
            let range = state.visible_range(list_capacity(anchor_row));
            let range_start = range.start;
            let visible_len = range.len();
            let selected = state.selected_window();
            let start = list_start_row(anchor_row, visible_len);
            let mut stdout = io::stdout();
            let mut entries = Vec::new();
            for (offset, window) in windows.into_iter().skip(range_start).take(visible_len).enumerate() {
                let row = start.saturating_add(offset as u16).saturating_add(1);
                let entry = terman_common::builtin_screen_control_windows_entry_hint(
                    window.index, window.active, &window.title, window.replay_bytes, attach_clients, cols, rows,
                );
                let entry = visible_entry(entry.as_str(), cols);
                entries.push((window.index, entry_width(&entry)));
                stdout.write_all(format!("\x1b[{row};1H\x1b[2K").as_bytes())?;
                if selected == Some(window.index) {
                    stdout.write_all(b"\x1b[7m")?;
                    stdout.write_all(entry.as_bytes())?;
                    stdout.write_all(b"\x1b[0m")?;
                } else {
                    stdout.write_all(entry.as_bytes())?;
                }
            }
            let status = visible_entry(&terman_common::builtin_screen_window_list_status_hint(), cols);
            stdout.write_all(format!("\x1b[{};1H\x1b[2K{}", anchor_row.saturating_add(1), status).as_bytes())?;
            stdout.flush()?;
            state.set_visible_entries(start, entries);
            Ok(())
        }
        ScreenIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::Unsupported, reason)),
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_screen_control_unexpected_response_hint(&format!("{response:?}")),
        )),
    }
}
fn select_list_window(
    endpoint: &ScreenIpcEndpoint,
    state: &mut AttachMouseState,
    row: u16,
    column: u16,
) -> io::Result<()> {
    let target = state.window_at(row, column);
    state.clear();
    match target {
        Some(index) => send_control_request(endpoint, ScreenIpcRequest::SelectWindow { index }),
        None => send_control_request(endpoint, ScreenIpcRequest::Redisplay),
    }
}

fn list_start_row(anchor_row: u16, len: usize) -> u16 {
    anchor_row.saturating_sub(len as u16)
}

fn visible_entry(entry: &str, cols: Option<u16>) -> String {
    match cols {
        Some(cols) => terman_common::truncate_terminal_text(entry, cols as usize),
        None => entry.to_string(),
    }
}

fn entry_width(entry: &str) -> u16 {
    terman_common::terminal_text_width(entry)
}

fn current_control_row() -> u16 {
    terman_common::current_terminal_size().map(|(_, rows)| rows).unwrap_or(1).saturating_sub(1)
}

fn list_capacity(anchor_row: u16) -> usize {
    usize::from(anchor_row.max(1))
}

fn on_control_row(row: u16) -> bool {
    terman_common::is_current_terminal_last_row(row)
}

fn forward_mouse_event(endpoint: &ScreenIpcEndpoint, event: MouseEvent) -> io::Result<()> {
    match mouse_event_bytes(event) {
        Some(bytes) => send_control_request(endpoint, ScreenIpcRequest::Input { bytes }),
        None => Ok(()),
    }
}

fn select_previous_window(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    send_control_request(endpoint, ScreenIpcRequest::PreviousWindow)
}

fn select_next_window(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    send_control_request(endpoint, ScreenIpcRequest::NextWindow)
}