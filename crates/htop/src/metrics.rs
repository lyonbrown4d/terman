use sysinfo::Users;
use sysinfo::{Networks, Signal, System};

use crate::{
    model::{CpuCore, LoadAverage, NetworkRow, Snapshot, SortMode, SystemSummary},
    network::socket_rows,
    process_priority,
    process_tree::ProcessTreeState,
    process_rows::{io_rows, process_rows},
};

pub(crate) struct Metrics {
    system: System,
    users: Users,
    networks: Networks,
}

impl Metrics {
    pub(crate) fn new() -> Self {
        Self {
            system: System::new_all(),
            users: Users::new_with_refreshed_list(),
            networks: Networks::new_with_refreshed_list(),
        }
    }

    pub(crate) fn refresh(&mut self) {
        self.system.refresh_all();
        self.networks.refresh(true);
    }

    pub(crate) fn process_exists(&self, pid: &str) -> bool {
        self.system
            .processes()
            .keys()
            .any(|candidate| candidate.to_string() == pid)
    }
    pub(crate) fn signal_process(&mut self, pid: &str, signal: Signal) -> bool {
        self.system
            .processes()
            .iter()
            .find(|(candidate, _)| candidate.to_string() == pid)
            .and_then(|(_, process)| process.kill_with(signal))
            .unwrap_or(false)
    }

    pub(crate) fn adjust_process_priority(&mut self, pid: &str, delta: i32) -> bool {
        process_priority::adjust(pid, delta).is_ok()
    }

    pub(crate) fn snapshot(
        &self,
        sort: SortMode,
        inverted: bool,
        filter: &str,
        user_filter: Option<&str>,
        tree: bool,
        tree_state: &ProcessTreeState,
    ) -> Snapshot {
        let networks = self.network_rows();
        let (processes, process_users) = process_rows(
            &self.system,
            &self.users,
            sort,
            inverted,
            filter,
            user_filter,
            tree,
            tree_state,
        );
        let io = io_rows(&self.system, &self.users, sort, inverted, filter, user_filter);
        let sockets = socket_rows(&self.system, sort, inverted);
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
            process_users,
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
            .map(|(index, cpu)| CpuCore { index, usage: cpu.cpu_usage() })
            .collect()
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

fn load_average() -> LoadAverage {
    let average = System::load_average();
    LoadAverage { one: average.one, five: average.five, fifteen: average.fifteen }
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
