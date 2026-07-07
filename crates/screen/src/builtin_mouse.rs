use std::io::{self, Write};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, MouseButton, MouseEvent, MouseEventKind},
    execute,
};

use crate::{
    builtin_output::publish_window_redraw,
    session_core::ScreenSessionBus,
    terminal_mouse::mouse_event_bytes,
    window_runtime::{ScreenWindowRuntime, ScreenWindowSwitch, switch_screen_window, write_active_window_input},
};

#[derive(Default)]
pub(crate) struct ScreenMouseState {
    window_list_start: Option<u16>,
    window_entries: Vec<(usize, u16)>,
}

impl ScreenMouseState {
    fn show_window_list(&mut self, start: u16, entries: Vec<(usize, u16)>) {
        self.window_list_start = Some(start);
        self.window_entries = entries;
    }

    fn clear(&mut self) {
        self.window_list_start = None;
        self.window_entries.clear();
    }

    fn list_open(&self) -> bool {
        self.window_list_start.is_some()
    }

    fn window_at(&self, row: u16, column: u16) -> Option<usize> {
        let start = self.window_list_start?;
        let offset = row.checked_sub(start)? as usize;
        let (index, width) = self.window_entries.get(offset).copied()?;
        (column < width).then_some(index)
    }
}

pub(crate) fn enable_mouse_capture() -> io::Result<()> {
    execute!(io::stdout(), EnableMouseCapture)
}

pub(crate) fn disable_mouse_capture() {
    let _ = execute!(io::stdout(), DisableMouseCapture);
}

pub(crate) fn handle_builtin_mouse(
    bus: &ScreenSessionBus,
    windows: &mut [ScreenWindowRuntime],
    active_window: &mut usize,
    state: &mut ScreenMouseState,
    event: MouseEvent,
) {
    match event.kind {
        MouseEventKind::ScrollUp | MouseEventKind::ScrollLeft => switch_with_mouse(bus, windows, active_window, state, ScreenWindowSwitch::Previous),
        MouseEventKind::ScrollDown | MouseEventKind::ScrollRight => switch_with_mouse(bus, windows, active_window, state, ScreenWindowSwitch::Next),
        MouseEventKind::Down(MouseButton::Left) => select_or_forward(bus, windows, active_window, state, event),
        MouseEventKind::Drag(MouseButton::Left) => select_or_forward(bus, windows, active_window, state, event),
        MouseEventKind::Down(MouseButton::Right) => publish_windows(bus, state, event.row),
        MouseEventKind::Down(MouseButton::Middle) => { state.clear(); publish_help(bus); }
        _ => forward_mouse_event(windows, *active_window, event),
    }
}

fn select_or_forward(
    bus: &ScreenSessionBus,
    windows: &mut [ScreenWindowRuntime],
    active_window: &mut usize,
    state: &mut ScreenMouseState,
    event: MouseEvent,
) {
    let list_open = state.list_open();
    if let Some(index) = state.window_at(event.row, event.column) {
        state.clear();
        if let Some(replay) = switch_screen_window(bus, windows, active_window, ScreenWindowSwitch::Select(index)) {
            publish_window_redraw(bus, &replay);
        }
        return;
    }
    state.clear();
    if !list_open {
        forward_mouse_event(windows, *active_window, event);
    }
}

fn forward_mouse_event(windows: &mut [ScreenWindowRuntime], active_window: usize, event: MouseEvent) {
    if let Some(bytes) = mouse_event_bytes(event) {
        write_active_window_input(windows, active_window, &bytes);
    }
}

fn switch_with_mouse(
    bus: &ScreenSessionBus,
    windows: &mut [ScreenWindowRuntime],
    active_window: &mut usize,
    state: &mut ScreenMouseState,
    target: ScreenWindowSwitch,
) {
    state.clear();
    if let Some(replay) = switch_screen_window(bus, windows, active_window, target) {
        publish_window_redraw(bus, &replay);
    }
}

fn publish_windows(bus: &ScreenSessionBus, state: &mut ScreenMouseState, anchor_row: u16) {
    let status = bus.status_snapshot();
    let attach_clients = status.attach_clients;
    let cols = status.cols;
    let rows = status.rows;
    let windows = status.windows;
    let start = list_start_row(anchor_row, rows, windows.len());
    let mut message = String::new();
    let mut entries = Vec::new();
    for (offset, window) in windows.into_iter().enumerate() {
        let row = start.saturating_add(offset as u16).saturating_add(1);
        let title = window.title.unwrap_or_else(|| format!("window-{}", window.index));
        let entry = terman_common::builtin_screen_control_windows_entry_hint(
            window.index, window.active, &title, window.replay_bytes, attach_clients, cols, rows,
        );
        entries.push((window.index, entry_width(entry.as_str())));
        message.push_str(&format!("\x1b[{row};1H\x1b[2K"));
        if window.active {
            message.push_str("\x1b[7m");
            message.push_str(&entry);
            message.push_str("\x1b[0m");
        } else {
            message.push_str(&entry);
        }
    }
    state.show_window_list(start, entries);
    publish_mouse_message(bus, message);
}

fn entry_width(entry: &str) -> u16 {
    entry.chars().count().min(u16::MAX as usize) as u16
}

fn list_start_row(anchor_row: u16, rows: Option<u16>, len: usize) -> u16 {
    let len = len.max(1) as u16;
    rows.map(|rows| anchor_row.min(rows.saturating_sub(len))).unwrap_or(anchor_row)
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