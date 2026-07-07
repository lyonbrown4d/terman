use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, MouseEvent, MouseEventKind},
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
    let target = match event.kind {
        MouseEventKind::ScrollUp => ScreenWindowSwitch::Previous,
        MouseEventKind::ScrollDown => ScreenWindowSwitch::Next,
        _ => return,
    };
    if let Some(replay) = switch_screen_window(bus, windows, active_window, target) {
        publish_window_redraw(bus, &replay);
    }
}