use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use sysinfo::{Networks, Process, System};

use crate::network::socket_rows;
use crate::model::{
    CpuCore, IoRow, LoadAverage, NetworkRow, ProcessRow, Snapshot, SortMode, SystemSummary,
};

pub(crate) struct Metrics {
    system: System,
    networks: Networks,
}

impl Metrics {
    pub(crate) fn new() -> Self {
        Self {
            system: System::new_all(),
            networks: Networks::new_with_refreshed_list(),
        }
    }

    pub(crate) fn refresh(&mut self) {
        self.system.refresh_all();
        self.networks.refresh(true);
    }

    pub(crate) fn kill_process(&mut self, pid: &str) -> bool {
        self.system
            .processes()
            .iter()
            .find(|(candidate, _)| candidate.to_string() == pid)
            .map(|(_, process)| process.kill())
            .unwrap_or(false)
    }

    pub(crate) fn snapshot(&self, sort: SortMode, filter: &str, tree: bool) -> Snapshot {
        let networks = self.network_rows();
        let processes = self.process_rows(sort, filter, tree);
        let io = self.io_rows(filter);
        let sockets = socket_rows(&self.system);
        let received = networks.iter().map(|row| row.received).sum();
        let transmitted = networks.iter().map(|row| row.transmitted).sum();
        Snapshot {
            cpu_usage: self.system.global_cpu_usage(),
            cpu_count: self.system.cpus().len(),
            cpu_cores: self.cpu_rows(),
            total_memory: self.system.total_memory(),
            used_memory: self.system.used_memory(),
            total_swap: self.system.total_swap(),
            used_swap: self.system.used_swap(),
            process_count: self.system.processes().len(),
            filtered_process_count: processes.len(),
            received_per_refresh: received,
            transmitted_per_refresh: transmitted,
            uptime: System::uptime(),
            load_average: load_average(),
            system: system_summary(),
            processes,
            io,
            networks,
            sockets,
        }
    }

    fn cpu_rows(&self) -> Vec<CpuCore> {
        self.system
            .cpus()
            .iter()
            .enumerate()
            .map(|(index, cpu)| CpuCore {
                index,
                usage: cpu.cpu_usage(),
            })
            .collect()
    }

    fn process_rows(&self, sort: SortMode, filter: &str, tree: bool) -> Vec<ProcessRow> {
        let mut rows: Vec<_> = self
            .system
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
            .filter(|row| process_matches(row.pid.as_str(), row.name.as_str(), filter))
            .collect();
        rows.sort_by(|left, right| compare_process(left, right, sort));
        if tree { tree_rows(rows) } else { rows }
    }

    fn io_rows(&self, filter: &str) -> Vec<IoRow> {
        let mut rows: Vec<_> = self
            .system
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
                }
            })
            .filter(|row| process_matches(row.pid.as_str(), row.name.as_str(), filter))
            .collect();
        rows.sort_by(|left, right| {
            (right.read_rate + right.written_rate)
                .cmp(&(left.read_rate + left.written_rate))
                .then_with(|| (right.read + right.written).cmp(&(left.read + left.written)))
        });
        rows
    }

    fn network_rows(&self) -> Vec<NetworkRow> {
        let mut rows = Vec::new();
        for (name, data) in &self.networks {
            rows.push(NetworkRow {
                name: name.to_string(),
                received: data.received(),
                transmitted: data.transmitted(),
                total_received: data.total_received(),
                total_transmitted: data.total_transmitted(),
            });
        }
        rows.sort_by(|left, right| {
            (right.received + right.transmitted).cmp(&(left.received + left.transmitted))
        });
        rows
    }
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
    if !seen.insert(row.pid.clone()) {
        return;
    }
    row.depth = depth;
    let pid = row.pid.clone();
    output.push(row);
    if let Some(child_rows) = children.remove(&pid) {
        for child in child_rows {
            append_tree(child, depth + 1, children, seen, output);
        }
    }
}

fn load_average() -> LoadAverage {
    let average = System::load_average();
    LoadAverage {
        one: average.one,
        five: average.five,
        fifteen: average.fifteen,
    }
}

fn system_summary() -> SystemSummary {
    SystemSummary {
        hostname: System::host_name().unwrap_or_else(|| "unknown".to_string()),
        os: System::long_os_version()
            .or_else(System::name)
            .unwrap_or_else(|| "unknown".to_string()),
        kernel: System::kernel_version().unwrap_or_else(|| "unknown".to_string()),
        arch: std::env::consts::ARCH.to_string(),
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

fn process_matches(pid: &str, name: &str, filter: &str) -> bool {
    let filter = filter.trim();
    filter.is_empty() || pid.contains(filter) || name.to_lowercase().contains(&filter.to_lowercase())
}

fn compare_process(left: &ProcessRow, right: &ProcessRow, sort: SortMode) -> Ordering {
    match sort {
        SortMode::Cpu => compare_f32(right.cpu, left.cpu),
        SortMode::Memory => right.memory.cmp(&left.memory),
        SortMode::Time => right.run_time.cmp(&left.run_time),
        SortMode::Io => compare_process_io(left, right),
        SortMode::Pid => left.pid.cmp(&right.pid),
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

fn compare_f32(left: f32, right: f32) -> Ordering {
    left.partial_cmp(&right).unwrap_or(Ordering::Equal)
}