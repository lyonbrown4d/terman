use std::io::{self, Write};

use crossterm::{
    cursor::{MoveTo, RestorePosition, SavePosition},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{size, Clear, ClearType},
};

use crate::{
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
};

pub(crate) const PREFIX_STATUS: &str = "tmux prefix | c new  x/& kill  n next  p previous  0-9 select  d detach";
pub(crate) const KILL_CONFIRM_STATUS: &str = "tmux confirm | kill current window? y yes  n/Esc no";

pub(crate) fn query_status_line(endpoint: &TmuxIpcEndpoint) -> io::Result<String> {
    match request_endpoint_response(endpoint, TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info { session_name, active_window, window_indexes, window_names, .. } => {
            let windows = window_indexes.iter().enumerate().map(|(position, index)| {
                let name = window_names.get(position).map(String::as_str).unwrap_or("-");
                if *index == active_window { format!("[{index}:{name}]") } else { format!("{index}:{name}") }
            }).collect::<Vec<_>>().join(" ");
            Ok(format!("tmux {session_name} | {windows} | C-b n/p/0-9 switch  C-b d detach"))
        }
        TmuxIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::PermissionDenied, reason)),
        response => Err(io::Error::new(io::ErrorKind::InvalidData, terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")))),
    }
}

pub(crate) fn render_status_line(status: &str) -> io::Result<()> {
    let (cols, rows) = size()?;
    let row = rows.saturating_sub(1);
    let text = fit_status_text(status, cols as usize);
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

fn fit_status_text(status: &str, width: usize) -> String {
    let mut text = status.chars().take(width).collect::<String>();
    let len = text.chars().count();
    if len < width { text.push_str(&" ".repeat(width - len)); }
    text
}