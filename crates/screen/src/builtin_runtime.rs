use std::{io, time::Duration};

use crossterm::{
    event::{self, Event},
    terminal,
};

use crate::{
    ScreenArgs,
    builtin_mouse::{ScreenMouseState, disable_mouse_capture, enable_mouse_capture, handle_builtin_mouse},
    ipc::ScreenIpcEndpoint,
    session_core::ScreenSessionBus,
    terminal_input::key_to_bytes,
    window_runtime::{ScreenWindowRuntime, resize_windows, write_active_window_input},
};

pub(crate) struct RawMode;

impl RawMode {
    pub(crate) fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        enable_mouse_capture()?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        disable_mouse_capture();
        let _ = terminal::disable_raw_mode();
    }
}

pub(crate) fn screen_session_endpoint(args: &ScreenArgs) -> ScreenIpcEndpoint {
    args.session_name
        .as_deref()
        .map(ScreenIpcEndpoint::for_new_session)
        .unwrap_or_else(|| ScreenIpcEndpoint::for_session("anonymous"))
}

pub(crate) fn resolve_size(cols_override: Option<u16>, rows_override: Option<u16>) -> (u16, u16) {
    let (cols, rows) = terman_common::current_terminal_size().unwrap_or((120, 32));
    (cols_override.unwrap_or(cols), rows_override.unwrap_or(rows))
}

pub(crate) fn poll_terminal_event(
    session_bus: &ScreenSessionBus,
    windows: &mut [ScreenWindowRuntime],
    active_window: &mut usize,
    mouse_state: &mut ScreenMouseState,
) -> io::Result<()> {
    if !event::poll(Duration::from_millis(16))? {
        return Ok(());
    }
    match event::read()? {
        Event::Mouse(mouse) => handle_builtin_mouse(session_bus, windows, active_window, mouse_state, mouse),
        Event::Key(key) => {
            if let Some(bytes) = key_to_bytes(key) {
                write_active_window_input(windows, *active_window, &bytes);
            }
        }
        Event::Resize(cols, rows) => {
            resize_windows(windows, cols, rows);
            session_bus.publish_resize(cols, rows);
        }
        _ => {}
    }
    Ok(())
}