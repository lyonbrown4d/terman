use std::io::{self, Write};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, MouseButton, MouseEvent, MouseEventKind},
    execute,
    terminal::size,
};

use super::{
    attach_output::print_attach_help,
    ipc_client::{request_endpoint_response, send_control_request},
};
use crate::{
    ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse},
    terminal_mouse::mouse_event_bytes,
};

#[derive(Default)]
pub(super) struct AttachMouseState {
    window_list_start: Option<u16>,
    window_entries: Vec<(usize, u16)>,
}

impl AttachMouseState {
    fn clear(&mut self) {
        self.window_list_start = None;
        self.window_entries.clear();
    }

    fn list_open(&self) -> bool {
        self.window_list_start.is_some()
    }

    fn show_window_list(&mut self, start: u16, entries: Vec<(usize, u16)>) {
        self.window_list_start = Some(start);
        self.window_entries = entries;
    }

    fn window_at(&self, row: u16, column: u16) -> Option<usize> {
        let start = self.window_list_start?;
        let offset = row.checked_sub(start)? as usize;
        let (index, width) = self.window_entries.get(offset).copied()?;
        (column < width).then_some(index)
    }
}

pub(super) fn enable_mouse_capture() -> io::Result<()> {
    execute!(io::stdout(), EnableMouseCapture)
}

pub(super) fn disable_mouse_capture() {
    let _ = execute!(io::stdout(), DisableMouseCapture);
}

pub(super) fn handle_attach_mouse(
    endpoint: &ScreenIpcEndpoint,
    state: &mut AttachMouseState,
    event: MouseEvent,
) -> io::Result<()> {
    if state.list_open()
        && matches!(event.kind, MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Up(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left))
    {
        return select_list_window(endpoint, state, event.row, event.column);
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
            let start = list_start_row(anchor_row, rows, windows.len());
            let mut stdout = io::stdout();
            let mut entries = Vec::new();
            for (offset, window) in windows.into_iter().enumerate() {
                let row = start.saturating_add(offset as u16).saturating_add(1);
                let entry = terman_common::builtin_screen_control_windows_entry_hint(
                    window.index, window.active, &window.title, window.replay_bytes, attach_clients, cols, rows,
                );
                entries.push((window.index, entry_width(&entry)));
                stdout.write_all(format!("\x1b[{row};1H\x1b[2K").as_bytes())?;
                if window.active {
                    stdout.write_all(b"\x1b[7m")?;
                    stdout.write_all(entry.as_bytes())?;
                    stdout.write_all(b"\x1b[0m")?;
                } else {
                    stdout.write_all(entry.as_bytes())?;
                }
            }
            stdout.flush()?;
            state.show_window_list(start, entries);
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
        None => Ok(()),
    }
}

fn list_start_row(anchor_row: u16, rows: Option<u16>, len: usize) -> u16 {
    let len = len.max(1) as u16;
    rows.map(|rows| anchor_row.min(rows.saturating_sub(len))).unwrap_or(anchor_row)
}

fn entry_width(entry: &str) -> u16 {
    entry.chars().count().min(u16::MAX as usize) as u16
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
