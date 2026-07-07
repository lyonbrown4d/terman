use std::io;

use crate::{
    attach_status::render_status_line,
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
};

pub(crate) fn render_window_list_status(endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
    let status = match request_endpoint_response(endpoint, TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info { active_window, window_indexes, window_names, .. } => {
            let windows = format_window_list(active_window, &window_indexes, &window_names);
            terman_common::builtin_tmux_attach_window_list(&windows)
        }
        TmuxIpcResponse::Rejected { reason } => {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, reason));
        }
        response => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")),
            ));
        }
    };
    render_status_line(&status)
}

fn format_window_list(active_window: u32, indexes: &[u32], names: &[String]) -> String {
    indexes
        .iter()
        .enumerate()
        .map(|(position, index)| format_window(*index, active_window, names.get(position)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_window(index: u32, active_window: u32, name: Option<&String>) -> String {
    let marker = if index == active_window { "*" } else { "" };
    let name = name.map(String::as_str).unwrap_or("window");
    format!("{index}:{name}{marker}")
}