use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, MouseButton, MouseEvent, MouseEventKind},
    execute,
    terminal::size,
};

use super::{
    attach_output::{print_attach_help, print_attach_windows},
    ipc_client::send_control_request,
};
use crate::{
    ipc::{ScreenIpcEndpoint, ScreenIpcRequest},
    terminal_mouse::mouse_event_bytes,
};

pub(super) fn enable_mouse_capture() -> io::Result<()> {
    execute!(io::stdout(), EnableMouseCapture)
}

pub(super) fn disable_mouse_capture() {
    let _ = execute!(io::stdout(), DisableMouseCapture);
}

pub(super) fn handle_attach_mouse(
    endpoint: &ScreenIpcEndpoint,
    event: MouseEvent,
) -> io::Result<()> {
    let control_row = on_control_row(event.row);
    match event.kind {
        MouseEventKind::ScrollUp | MouseEventKind::ScrollLeft if control_row => {
            select_previous_window(endpoint)
        }
        MouseEventKind::ScrollDown | MouseEventKind::ScrollRight if control_row => {
            select_next_window(endpoint)
        }
        MouseEventKind::Down(MouseButton::Right) if control_row => print_attach_windows(endpoint),
        MouseEventKind::Down(MouseButton::Middle) if control_row => print_attach_help(),
        MouseEventKind::Up(MouseButton::Right) | MouseEventKind::Up(MouseButton::Middle)
            if control_row => Ok(()),
        _ => forward_mouse_event(endpoint, event),
    }
}

fn on_control_row(row: u16) -> bool {
    size()
        .map(|(_, rows)| row == rows.saturating_sub(1))
        .unwrap_or(false)
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
