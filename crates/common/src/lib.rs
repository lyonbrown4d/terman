use std::env;

use which::which;

pub fn which_binary(name: &str) -> Option<String> {
    which(name).ok().map(|path| path.to_string_lossy().to_string())
}

pub fn passthrough_env() -> Vec<(String, String)> {
    [
        "TERM",
        "COLORTERM",
        "LC_ALL",
        "LANG",
        "LC_CTYPE",
        "TERM_PROGRAM",
        "TERM_PROGRAM_VERSION",
    ]
    .iter()
    .filter_map(|k| env::var(k).ok().map(|v| (k.to_string(), v)))
    .collect()
}

pub fn wsl_install_hint(tool: &str) -> String {
    format!(
        "建议先在 WSL 内执行 `wsl -e which {tool}` / `wsl -e {tool} -V` 确认安装与版本。"
    )
}

pub fn wsl_runtime_hint(tool: &str) -> String {
    format!(
        "建议先执行 `wsl -l -v`（检查发行版）、`wsl --status`（检查子系统）与 `wsl -e {tool} -V`（确认 {tool} 可用）。"
    )
}
