use super::{MessageKey, localized_message};

pub fn builtin_htop_overview_cpu_hint(usage: f32, cores: usize) -> String {
    let usage = format!("{usage:.1}");
    let cores = cores.to_string();
    localized_message(
        MessageKey::BuiltinHtopOverviewCpu,
        &[("usage", &usage), ("cores", &cores)],
    )
}

pub fn builtin_htop_overview_host_hint(host: &str, os: &str) -> String {
    localized_message(
        MessageKey::BuiltinHtopOverviewHost,
        &[("host", host), ("os", os)],
    )
}

pub fn builtin_htop_overview_kernel_hint(kernel: &str, arch: &str) -> String {
    localized_message(
        MessageKey::BuiltinHtopOverviewKernel,
        &[("kernel", kernel), ("arch", arch)],
    )
}

pub fn builtin_htop_overview_tasks_hint(shown: usize, total: usize) -> String {
    let shown = shown.to_string();
    let total = total.to_string();
    localized_message(
        MessageKey::BuiltinHtopOverviewTasks,
        &[("shown", &shown), ("total", &total)],
    )
}

pub fn builtin_htop_overview_states_hint(
    running: usize,
    sleeping: usize,
    stopped: usize,
    zombie: usize,
    other: usize,
) -> String {
    let running = running.to_string();
    let sleeping = sleeping.to_string();
    let stopped = stopped.to_string();
    let zombie = zombie.to_string();
    let other = other.to_string();
    localized_message(
        MessageKey::BuiltinHtopOverviewStates,
        &[
            ("running", &running),
            ("sleeping", &sleeping),
            ("stopped", &stopped),
            ("zombie", &zombie),
            ("other", &other),
        ],
    )
}

pub fn builtin_htop_overview_network_hint(rx: &str, tx: &str) -> String {
    localized_message(
        MessageKey::BuiltinHtopOverviewNetwork,
        &[("rx", rx), ("tx", tx)],
    )
}

pub fn builtin_htop_overview_uptime_hint(uptime: &str) -> String {
    localized_message(
        MessageKey::BuiltinHtopOverviewUptime,
        &[("uptime", uptime)],
    )
}

pub fn builtin_htop_overview_load_hint(
    one: f64,
    five: f64,
    fifteen: f64,
) -> String {
    let one = format!("{one:.2}");
    let five = format!("{five:.2}");
    let fifteen = format!("{fifteen:.2}");
    localized_message(
        MessageKey::BuiltinHtopOverviewLoad,
        &[("one", &one), ("five", &five), ("fifteen", &fifteen)],
    )
}

pub fn builtin_htop_overview_top_processes_hint() -> String {
    localized_message(MessageKey::BuiltinHtopOverviewTopProcesses, &[])
}

pub fn builtin_htop_detail_none_hint() -> String {
    localized_message(MessageKey::BuiltinHtopDetailNone, &[])
}

pub fn builtin_htop_detail_user_hint() -> String {
    localized_message(MessageKey::BuiltinHtopDetailUser, &[])
}

pub fn builtin_htop_detail_status_hint() -> String {
    localized_message(MessageKey::BuiltinHtopDetailStatus, &[])
}

pub fn builtin_htop_detail_memory_hint() -> String {
    localized_message(MessageKey::BuiltinHtopDetailMemory, &[])
}

pub fn builtin_htop_detail_runtime_hint() -> String {
    localized_message(MessageKey::BuiltinHtopDetailRuntime, &[])
}

pub fn builtin_htop_detail_read_hint() -> String {
    localized_message(MessageKey::BuiltinHtopDetailRead, &[])
}

pub fn builtin_htop_detail_write_hint() -> String {
    localized_message(MessageKey::BuiltinHtopDetailWrite, &[])
}

pub fn builtin_htop_detail_command_hint() -> String {
    localized_message(MessageKey::BuiltinHtopDetailCommand, &[])
}

pub fn builtin_htop_detail_io_hint(rate: &str, total: &str) -> String {
    localized_message(
        MessageKey::BuiltinHtopDetailIo,
        &[("rate", rate), ("total", total)],
    )
}
pub fn builtin_htop_environment_title_hint(pid: &str) -> String {
    localized_message(
        MessageKey::BuiltinHtopEnvironmentTitle,
        &[("pid", pid)],
    )
}

pub fn builtin_htop_environment_help_hint() -> String {
    localized_message(MessageKey::BuiltinHtopEnvironmentHelp, &[])
}

pub fn builtin_htop_environment_empty_hint() -> String {
    localized_message(MessageKey::BuiltinHtopEnvironmentEmpty, &[])
}
pub fn builtin_htop_processes_title_hint() -> String {
    localized_message(MessageKey::BuiltinHtopProcessesTitle, &[])
}

pub fn builtin_htop_processes_details_hint() -> String {
    localized_message(MessageKey::BuiltinHtopProcessesDetails, &[])
}

pub fn builtin_htop_processes_status_hint(
    sort: &str,
    view: &str,
    selection: &str,
    filter: &str,
) -> String {
    localized_message(
        MessageKey::BuiltinHtopProcessesStatus,
        &[("sort", sort), ("view", view), ("selection", selection), ("filter", filter)],
    )
}

pub fn builtin_htop_processes_view_tree_hint() -> String {
    localized_message(MessageKey::BuiltinHtopProcessesViewTree, &[])
}

pub fn builtin_htop_processes_view_flat_hint() -> String {
    localized_message(MessageKey::BuiltinHtopProcessesViewFlat, &[])
}
