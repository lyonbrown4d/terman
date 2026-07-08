use std::io;

use crate::{
    attach_status::render_status_line,
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
};

#[derive(Clone, Debug, Default)]
pub(crate) struct TmuxWindowListLayout {
    hits: Vec<TmuxWindowListHit>,
}

#[derive(Clone, Debug)]
struct TmuxWindowListHit {
    index: u32,
    start: u16,
    end: u16,
}

impl TmuxWindowListLayout {
    pub(crate) fn window_at(&self, column: u16) -> Option<u32> {
        self.hits.iter().find(|hit| column >= hit.start && column < hit.end).map(|hit| hit.index)
    }
}

pub(crate) fn render_window_list_status(endpoint: &TmuxIpcEndpoint) -> io::Result<TmuxWindowListLayout> {
    match request_endpoint_response(endpoint, TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info { active_window, window_indexes, window_names, .. } => {
            let (status, layout) = format_window_list_status(active_window, &window_indexes, &window_names);
            render_status_line(&status)?;
            Ok(layout)
        }
        TmuxIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::PermissionDenied, reason)),
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")),
        )),
    }
}

fn format_window_list_status(active_window: u32, indexes: &[u32], names: &[String]) -> (String, TmuxWindowListLayout) {
    let labels = window_labels(active_window, indexes, names);
    let windows = labels.iter().map(|(_, label)| label.as_str()).collect::<Vec<_>>().join(" ");
    let status = terman_common::builtin_tmux_attach_window_list(&windows);
    let layout = layout_window_labels(&status, &labels);
    (status, layout)
}

fn window_labels(active_window: u32, indexes: &[u32], names: &[String]) -> Vec<(u32, String)> {
    indexes.iter().enumerate().map(|(position, index)| {
        (*index, format_window(*index, active_window, names.get(position)))
    }).collect()
}

fn layout_window_labels(status: &str, labels: &[(u32, String)]) -> TmuxWindowListLayout {
    let mut hits = Vec::new();
    let mut search_byte = 0;
    for (index, label) in labels {
        let Some(relative) = status[search_byte..].find(label) else { continue; };
        let start_byte = search_byte + relative;
        let end_byte = start_byte + label.len();
        let start = terman_common::terminal_text_width(&status[..start_byte]);
        let width = terman_common::terminal_text_width(label.as_str());
        hits.push(TmuxWindowListHit { index: *index, start, end: start.saturating_add(width) });
        search_byte = end_byte;
    }
    TmuxWindowListLayout { hits }
}

fn format_window(index: u32, active_window: u32, name: Option<&String>) -> String {
    let name = name.map(String::as_str).unwrap_or("window");
    if index == active_window { format!("[{index}:{name}]") } else { format!("{index}:{name}") }
}
