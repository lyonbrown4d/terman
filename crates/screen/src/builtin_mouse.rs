use std::io::{self, Write};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, MouseButton, MouseEvent, MouseEventKind},
    execute,
};

use crate::{
    builtin_output::publish_window_redraw,
    session_core::ScreenSessionBus,
    window_runtime::{ScreenWindowRuntime, ScreenWindowSwitch, switch_screen_window},
};

pub(crate) fn enable_mouse_capture() -> io::Result<()> {
    execute!(io::stdout(), EnableMouseCapture)
}

pub(crate) fn disable_mouse_capture() {
    let _ = execute!(io::stdout(), DisableMouseCapture);
}

pub(crate) fn handle_builtin_mouse(
    bus: &ScreenSessionBus,
    windows: &[ScreenWindowRuntime],
    active_window: &mut usize,
    event: MouseEvent,
) {
    match event.kind {
        MouseEventKind::ScrollUp => switch_with_mouse(bus, windows, active_window, ScreenWindowSwitch::Previous),
        MouseEventKind::ScrollDown => switch_with_mouse(bus, windows, active_window, ScreenWindowSwitch::Next),
        MouseEventKind::Down(MouseButton::Right) => publish_windows(bus),
        MouseEventKind::Down(MouseButton::Middle) => publish_help(bus),
        _ => {}
    }
}

fn switch_with_mouse(
    bus: &ScreenSessionBus,
    windows: &[ScreenWindowRuntime],
    active_window: &mut usize,
    target: ScreenWindowSwitch,
) {
    if let Some(replay) = switch_screen_window(bus, windows, active_window, target) {
        publish_window_redraw(bus, &replay);
    }
}

fn publish_windows(bus: &ScreenSessionBus) {
    let status = bus.status_snapshot();
    let mut message = String::from("\r\n");
    for window in status.windows {
        let title = window
            .title
            .unwrap_or_else(|| format!("window-{}", window.index));
        message.push_str(&terman_common::builtin_screen_control_windows_entry_hint(
            window.index,
            window.active,
            &title,
            window.replay_bytes,
            status.attach_clients,
            status.cols,
            status.rows,
        ));
        message.push_str("\r\n");
    }
    publish_mouse_message(bus, message);
}

fn publish_help(bus: &ScreenSessionBus) {
    let mut message = String::from("\r\n");
    message.push_str(&terman_common::builtin_screen_attach_help_hint());
    message.push_str("\r\n");
    publish_mouse_message(bus, message);
}

fn publish_mouse_message(bus: &ScreenSessionBus, message: String) {
    bus.publish_transient_output(message.as_bytes());
    let mut stdout = io::stdout();
    let _ = stdout.write_all(message.as_bytes());
    let _ = stdout.flush();
}