use crate::{body_layout, model::Snapshot, render::Tab};

pub(crate) fn selected_data_index(tab: Tab, snapshot: &Snapshot, selected: usize) -> Option<usize> {
    let pid = snapshot.processes.get(selected)?.pid.as_str();
    match tab {
        Tab::Io => snapshot.io.iter().position(|row| row.pid == pid),
        Tab::Network => snapshot.sockets.iter().position(|row| row.pid == pid),
        _ => None,
    }
}
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
    let visible = body_layout::terminal_data_rows();
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
    let data_rows = body_layout::terminal_data_rows();
    let interfaces = body_layout::network_interface_rows(data_rows, sockets);
    body_layout::network_connection_rows(data_rows, interfaces)
}