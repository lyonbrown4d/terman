use std::cmp::Ordering;

use sysinfo::{Networks, System};

pub(crate) struct Metrics {
    system: System,
    networks: Networks,
}

#[derive(Clone, Debug)]
pub(crate) struct Snapshot {
    pub(crate) total_memory: u64,
    pub(crate) used_memory: u64,
    pub(crate) processes: Vec<ProcessRow>,
    pub(crate) io: Vec<IoRow>,
    pub(crate) networks: Vec<NetworkRow>,
}

#[derive(Clone, Debug)]
pub(crate) struct ProcessRow {
    pub(crate) pid: String,
    pub(crate) cpu: f32,
    pub(crate) memory: u64,
    pub(crate) name: String,
}

#[derive(Clone, Debug)]
pub(crate) struct IoRow {
    pub(crate) pid: String,
    pub(crate) read: u64,
    pub(crate) written: u64,
    pub(crate) name: String,
}

#[derive(Clone, Debug)]
pub(crate) struct NetworkRow {
    pub(crate) name: String,
    pub(crate) received: u64,
    pub(crate) transmitted: u64,
    pub(crate) total_received: u64,
    pub(crate) total_transmitted: u64,
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

    pub(crate) fn snapshot(&self) -> Snapshot {
        Snapshot {
            total_memory: self.system.total_memory(),
            used_memory: self.system.used_memory(),
            processes: self.process_rows(),
            io: self.io_rows(),
            networks: self.network_rows(),
        }
    }

    fn process_rows(&self) -> Vec<ProcessRow> {
        let mut rows: Vec<_> = self.system.processes().iter().map(|(pid, process)| {
            ProcessRow {
                pid: pid.to_string(),
                cpu: process.cpu_usage(),
                memory: process.memory(),
                name: process.name().to_string_lossy().into_owned(),
            }
        }).collect();
        rows.sort_by(|left, right| compare_f32(right.cpu, left.cpu));
        rows
    }

    fn io_rows(&self) -> Vec<IoRow> {
        let mut rows: Vec<_> = self.system.processes().iter().map(|(pid, process)| {
            let usage = process.disk_usage();
            IoRow {
                pid: pid.to_string(),
                read: usage.total_read_bytes,
                written: usage.total_written_bytes,
                name: process.name().to_string_lossy().into_owned(),
            }
        }).collect();
        rows.sort_by(|left, right| (right.read + right.written).cmp(&(left.read + left.written)));
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

fn compare_f32(left: f32, right: f32) -> Ordering {
    left.partial_cmp(&right).unwrap_or(Ordering::Equal)
}