use std::io::{self, Write};
use std::sync::Mutex;

use crossterm::{
    cursor::{MoveTo, RestorePosition, SavePosition},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};

use crate::{
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
};

pub(crate) fn kill_pane_confirm_status() -> String {
    terman_common::builtin_tmux_kill_pane_confirm_hint()
}
pub(crate) fn kill_window_confirm_status() -> String {
    terman_common::builtin_tmux_kill_window_confirm_hint()
}

pub(crate) fn query_status_line(endpoint: &TmuxIpcEndpoint) -> io::Result<String> {
    match request_endpoint_response(endpoint, TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info { session_name, active_window, window_indexes, window_names, .. } => {
            let windows = window_indexes.iter().enumerate().map(|(position, index)| {
                let name = window_names.get(position).map(String::as_str).unwrap_or("-");
                if *index == active_window { format!("[{index}:{name}]") } else { format!("{index}:{name}") }
            }).collect::<Vec<_>>().join(" ");
            Ok(terman_common::builtin_tmux_status_line_hint(&session_name, &windows))
        }
        TmuxIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::PermissionDenied, reason)),
        response => Err(io::Error::new(io::ErrorKind::InvalidData, terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")))),
    }
}

pub(crate) fn render_status_line(status: &str) -> io::Result<()> {
    let (cols, rows) = terman_common::current_terminal_size()?;
    let row = rows.saturating_sub(1);
    let text = terman_common::fit_terminal_text(status, cols as usize);
    let mut stdout = io::stdout().lock();
    execute!(
        stdout,
        SavePosition,
        MoveTo(0, row),
        SetBackgroundColor(Color::DarkBlue),
        SetForegroundColor(Color::White),
        Clear(ClearType::CurrentLine),
        Print(text),
        ResetColor,
        RestorePosition
    )?;
    stdout.flush()
}
pub(crate) fn render_status_line_with_override(
    status: &str,
    status_override: &Mutex<Option<String>>,
) -> io::Result<()> {
    let override_status = status_override
        .lock()
        .ok()
        .and_then(|value| value.clone());
    render_status_line(override_status.as_deref().unwrap_or(status))
}
