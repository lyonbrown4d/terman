use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, MouseButton, MouseEvent, MouseEventKind},
    execute,
};

use super::{
    attach_output::{print_attach_help, print_attach_windows},
    ipc_client::send_control_request,
};
use crate::ipc::{ScreenIpcEndpoint, ScreenIpcRequest};

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
    match event.kind {
        MouseEventKind::ScrollUp => select_previous_window(endpoint),
        MouseEventKind::ScrollDown => select_next_window(endpoint),
        MouseEventKind::Down(MouseButton::Right) => print_attach_windows(endpoint),
        MouseEventKind::Down(MouseButton::Middle) => print_attach_help(),
        _ => Ok(()),
    }
}

fn select_previous_window(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    send_control_request(endpoint, ScreenIpcRequest::PreviousWindow)
}

fn select_next_window(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    send_control_request(endpoint, ScreenIpcRequest::NextWindow)
}