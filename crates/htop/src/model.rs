#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SortMode {
    Cpu,
    Memory,
    Time,
    Io,
    Pid,
    ParentPid,
    State,
    Name,
}

impl SortMode {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Cpu => "CPU",
            Self::Memory => "MEM",
            Self::Time => "TIME",
            Self::Io => "I/O",
            Self::Pid => "PID",
            Self::ParentPid => "PPID",
            Self::State => "STATE",
            Self::Name => "NAME",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Snapshot {
    pub(crate) cpu_usage: f32,
    pub(crate) cpu_count: usize,
    pub(crate) cpu_cores: Vec<CpuCore>,
    pub(crate) total_memory: u64,
    pub(crate) used_memory: u64,
    pub(crate) total_swap: u64,
    pub(crate) used_swap: u64,
    pub(crate) process_count: usize,
    pub(crate) filtered_process_count: usize,
    pub(crate) received_per_refresh: u64,
    pub(crate) transmitted_per_refresh: u64,
    pub(crate) uptime: u64,
    pub(crate) load_average: LoadAverage,
    pub(crate) system: SystemSummary,
    pub(crate) processes: Vec<ProcessRow>,
    pub(crate) io: Vec<IoRow>,
    pub(crate) networks: Vec<NetworkRow>,
    pub(crate) sockets: Vec<SocketRow>,
}

#[derive(Clone, Debug)]
pub(crate) struct LoadAverage {
    pub(crate) one: f64,
    pub(crate) five: f64,
    pub(crate) fifteen: f64,
}

#[derive(Clone, Debug)]
pub(crate) struct SystemSummary {
    pub(crate) hostname: String,
    pub(crate) os: String,
    pub(crate) kernel: String,
    pub(crate) arch: String,
}

#[derive(Clone, Debug)]
pub(crate) struct CpuCore {
    pub(crate) index: usize,
    pub(crate) usage: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct ProcessRow {
    pub(crate) pid: String,
    pub(crate) parent_pid: Option<String>,
    pub(crate) depth: usize,
    pub(crate) status: String,
    pub(crate) run_time: u64,
    pub(crate) command: String,
    pub(crate) cpu: f32,
    pub(crate) memory: u64,
    pub(crate) read_rate: u64,
    pub(crate) written_rate: u64,
    pub(crate) read_total: u64,
    pub(crate) written_total: u64,
    pub(crate) name: String,
}

#[derive(Clone, Debug)]
pub(crate) struct IoRow {
    pub(crate) pid: String,
    pub(crate) read_rate: u64,
    pub(crate) written_rate: u64,
    pub(crate) read: u64,
    pub(crate) written: u64,
    pub(crate) name: String,
    pub(crate) command: String,
}

#[derive(Clone, Debug)]
pub(crate) struct NetworkRow {
    pub(crate) name: String,
    pub(crate) received: u64,
    pub(crate) transmitted: u64,
    pub(crate) total_received: u64,
    pub(crate) total_transmitted: u64,
}
#[derive(Clone, Debug)]
pub(crate) struct SocketRow {
    pub(crate) protocol: String,
    pub(crate) local: String,
    pub(crate) remote: String,
    pub(crate) state: String,
    pub(crate) pid: String,
    pub(crate) process: String,
}