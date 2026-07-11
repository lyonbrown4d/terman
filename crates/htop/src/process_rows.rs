use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use sysinfo::{Process, System};

use crate::model::{IoRow, ProcessRow, SortMode};

pub(crate) fn process_rows(system: &System, sort: SortMode, inverted: bool, filter: &str, tree: bool) -> Vec<ProcessRow> {
    let mut rows: Vec<_> = system
        .processes()
        .iter()
        .map(|(pid, process)| {
            let usage = process.disk_usage();
            ProcessRow {
                pid: pid.to_string(),
                parent_pid: process.parent().map(|parent| parent.to_string()),
                depth: 0,
                status: format!("{:?}", process.status()),
                run_time: process.run_time(),
                command: command_line(process),
                cpu: process.cpu_usage(),
                memory: process.memory(),
                read_rate: usage.read_bytes,
                written_rate: usage.written_bytes,
                read_total: usage.total_read_bytes,
                written_total: usage.total_written_bytes,
                name: process.name().to_string_lossy().into_owned(),
            }
        })
        .filter(|row| process_matches(row.pid.as_str(), row.name.as_str(), row.command.as_str(), filter))
        .collect();
    rows.sort_by(|left, right| compare_process(left, right, sort));
    if inverted { rows.reverse(); }
    if tree { tree_rows(rows) } else { rows }
}

pub(crate) fn io_rows(system: &System, sort: SortMode, inverted: bool, filter: &str) -> Vec<IoRow> {
    let mut rows: Vec<_> = system
        .processes()
        .iter()
        .map(|(pid, process)| {
            let usage = process.disk_usage();
            IoRow {
                pid: pid.to_string(),
                read_rate: usage.read_bytes,
                written_rate: usage.written_bytes,
                read: usage.total_read_bytes,
                written: usage.total_written_bytes,
                name: process.name().to_string_lossy().into_owned(),
                command: command_line(process),
            }
        })
        .filter(|row| process_matches(row.pid.as_str(), row.name.as_str(), row.command.as_str(), filter))
        .collect();
    rows.sort_by(|left, right| compare_io_row(left, right, sort));
    if inverted { rows.reverse(); }
    rows
}

fn tree_rows(rows: Vec<ProcessRow>) -> Vec<ProcessRow> {
    let included: HashSet<_> = rows.iter().map(|row| row.pid.clone()).collect();
    let mut roots = Vec::new();
    let mut children: HashMap<String, Vec<ProcessRow>> = HashMap::new();
    for row in rows {
        if let Some(parent) = row.parent_pid.as_ref().filter(|parent| included.contains(*parent)) {
            children.entry(parent.clone()).or_default().push(row);
        } else {
            roots.push(row);
        }
    }
    let mut output = Vec::new();
    let mut seen = HashSet::new();
    for root in roots {
        append_tree(root, 0, &mut children, &mut seen, &mut output);
    }
    output
}

fn append_tree(
    mut row: ProcessRow,
    depth: usize,
    children: &mut HashMap<String, Vec<ProcessRow>>,
    seen: &mut HashSet<String>,
    output: &mut Vec<ProcessRow>,
) {
    if !seen.insert(row.pid.clone()) { return; }
    row.depth = depth;
    let pid = row.pid.clone();
    output.push(row);
    if let Some(child_rows) = children.remove(&pid) {
        for child in child_rows {
            append_tree(child, depth + 1, children, seen, output);
        }
    }
}

fn command_line(process: &Process) -> String {
    let parts = process.cmd();
    if parts.is_empty() {
        return process.name().to_string_lossy().into_owned();
    }
    parts
        .iter()
        .map(|part| part.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(" ")
}

fn process_matches(pid: &str, name: &str, command: &str, filter: &str) -> bool {
    let filter = filter.trim();
    let lowered = filter.to_lowercase();
    filter.is_empty()
        || pid.contains(filter)
        || name.to_lowercase().contains(&lowered)
        || command.to_lowercase().contains(&lowered)
}

fn compare_process(left: &ProcessRow, right: &ProcessRow, sort: SortMode) -> Ordering {
    match sort {
        SortMode::Cpu => compare_f32(right.cpu, left.cpu),
        SortMode::Memory => right.memory.cmp(&left.memory),
        SortMode::Time => right.run_time.cmp(&left.run_time),
        SortMode::Io => compare_process_io(left, right),
        SortMode::Pid => left.pid.cmp(&right.pid),
        SortMode::ParentPid => left.parent_pid.as_deref().unwrap_or("")
            .cmp(right.parent_pid.as_deref().unwrap_or(""))
            .then_with(|| left.pid.cmp(&right.pid)),
        SortMode::State => left.status.cmp(&right.status).then_with(|| left.pid.cmp(&right.pid)),
        SortMode::Name => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
    }
}

fn compare_process_io(left: &ProcessRow, right: &ProcessRow) -> Ordering {
    let left_rate = left.read_rate + left.written_rate;
    let right_rate = right.read_rate + right.written_rate;
    let left_total = left.read_total + left.written_total;
    let right_total = right.read_total + right.written_total;
    right_rate.cmp(&left_rate).then_with(|| right_total.cmp(&left_total))
}

fn compare_io_row(left: &IoRow, right: &IoRow, sort: SortMode) -> Ordering {
    match sort {
        SortMode::Pid => left.pid.cmp(&right.pid),
        SortMode::ParentPid | SortMode::State => left.pid.cmp(&right.pid),
        SortMode::Name => left.name.to_lowercase().cmp(&right.name.to_lowercase()),
        _ => (right.read_rate + right.written_rate)
            .cmp(&(left.read_rate + left.written_rate))
            .then_with(|| (right.read + right.written).cmp(&(left.read + left.written))),
    }
}

fn compare_f32(left: f32, right: f32) -> Ordering {
    left.partial_cmp(&right).unwrap_or(Ordering::Equal)
}