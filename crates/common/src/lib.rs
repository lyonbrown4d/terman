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

pub fn which_wsl_binary() -> Option<String> {
    which_binary("wsl").or_else(|| which_binary("wsl.exe"))
}

pub fn wsl_install_hint(tool: &str) -> String {
    format!(
        "建议先在 WSL 内执行 `wsl -e which {tool}` / `wsl -e {tool} -V` 确认安装与版本。"
    )
}

pub fn wsl_precheck_not_found_hint(tool: &str) -> String {
    format!(
        "当前已进入 WSL 回退路径，但未检测到 WSL 内 {tool}。请先安装：wsl -e sudo apt install {tool}。"
    )
}

pub fn wsl_runtime_hint(tool: &str) -> String {
    format!(
        "建议先执行 `wsl -l -v`（检查发行版）、`wsl --status`（检查子系统）与 `wsl -e {tool} -V`（确认 {tool} 可用）。"
    )
}

#[cfg(test)]
mod tests {
    use super::wsl_precheck_not_found_hint;

    #[test]
    fn wsl_precheck_not_found_hint_mentions_tool_and_install_cmd() {
        let tool = "tmux";
        let hint = wsl_precheck_not_found_hint(tool);

        assert!(hint.contains("未检测到 WSL 内 tmux"));
        assert!(hint.contains("wsl -e sudo apt install tmux"));
    }
}
