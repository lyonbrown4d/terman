use crossterm::terminal;

use crate::{model::Snapshot, render::Tab};

pub(crate) fn keep_selected_visible(
    tab: Tab,
    snapshot: &Snapshot,
    selected: usize,
    io_scroll: &mut usize,
    network_scroll: &mut usize,
) {
    let Some(pid) = snapshot.processes.get(selected).map(|row| row.pid.as_str()) else {
        return;
    };
    match tab {
        Tab::Io => keep_io_visible(snapshot, pid, io_scroll),
        Tab::Network => keep_network_visible(snapshot, pid, network_scroll),
        _ => {}
    }
}

fn keep_io_visible(snapshot: &Snapshot, pid: &str, scroll: &mut usize) {
    let visible = body_rows();
    let max = snapshot.io.len().saturating_sub(visible);
    *scroll = (*scroll).min(max);
    if let Some(index) = snapshot.io.iter().position(|row| row.pid == pid) {
        *scroll = visible_scroll(*scroll, index, visible, snapshot.io.len());
    }
}

fn keep_network_visible(snapshot: &Snapshot, pid: &str, scroll: &mut usize) {
    if snapshot.sockets.is_empty() {
        return;
    }
    let visible = connection_rows(snapshot.sockets.len());
    let max = snapshot.sockets.len().saturating_sub(visible);
    *scroll = (*scroll).min(max);
    if let Some(index) = snapshot.sockets.iter().position(|row| row.pid == pid) {
        *scroll = visible_scroll(*scroll, index, visible, snapshot.sockets.len());
    }
}

fn visible_scroll(current: usize, index: usize, visible: usize, total: usize) -> usize {
    if visible == 0 || total <= visible {
        0
    } else if index < current {
        index
    } else if index >= current + visible {
        (index + 1 - visible).min(total - visible)
    } else {
        current
    }
}

fn connection_rows(sockets: usize) -> usize {
    if sockets == 0 {
        return 0;
    }
    let body = body_rows();
    let interfaces = 4usize.min(body.saturating_sub(6));
    body.saturating_sub(interfaces + 4)
}

fn body_rows() -> usize {
    terminal::size()
        .map(|(_, rows)| rows.saturating_sub(10) as usize)
        .unwrap_or(14)
}