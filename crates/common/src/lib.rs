use std::{
    env,
    ffi::OsString,
    io,
    process::{ExitStatus, Stdio},
    time::Duration,
};

use tokio::process::Command as TokioCommand;
use which::which;

pub const DEFAULT_COMMAND_TIMEOUT: Duration = Duration::from_secs(8);

pub fn command_status_with_timeout(
    command: &str,
    args: &[&str],
    timeout: Duration,
) -> io::Result<Option<ExitStatus>> {
    let command = command.to_string();
    let args: Vec<OsString> = args.iter().map(OsString::from).collect();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()?;

    runtime.block_on(command_status_with_timeout_async(command, args, timeout))
}

pub async fn command_status_with_timeout_async(
    command: String,
    args: Vec<OsString>,
    timeout: Duration,
) -> io::Result<Option<ExitStatus>> {
    let mut child = TokioCommand::new(command)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()?;

    match tokio::time::timeout(timeout, child.wait()).await {
        Ok(status) => status.map(Some),
        Err(_) => Ok(None),
    }
}

pub fn wsl_which_status_with_timeout(
    wsl_command: &str,
    tool: &str,
    timeout: Duration,
) -> io::Result<Option<ExitStatus>> {
    command_status_with_timeout(wsl_command, &["-e", "which", tool], timeout)
}
pub fn which_binary(name: &str) -> Option<String> {
    which(name)
        .ok()
        .map(|path| path.to_string_lossy().to_string())
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
    format!("建议先在 WSL 内执行 `wsl -e which {tool}` / `wsl -e {tool} -V` 确认安装与版本。")
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
    use super::{command_status_with_timeout, wsl_precheck_not_found_hint, wsl_runtime_hint};
    use std::time::Duration;

    #[test]
    fn wsl_precheck_not_found_hint_mentions_tool_and_install_cmd() {
        let tool = "tmux";
        let hint = wsl_precheck_not_found_hint(tool);

        assert!(hint.contains("未检测到 WSL 内 tmux"));
        assert!(hint.contains("wsl -e sudo apt install tmux"));
    }

    #[test]
    fn wsl_runtime_hint_uses_wsl_version_check_for_tool() {
        let tool = "screen";
        let hint = wsl_runtime_hint(tool);

        assert!(hint.contains("wsl -l -v"));
        assert!(hint.contains("wsl --status"));
        assert!(hint.contains("wsl -e screen -V"));
    }

    #[test]
    fn command_status_with_timeout_returns_status_for_successful_command() {
        let status = if cfg!(windows) {
            command_status_with_timeout("cmd", &["/C", "exit 0"], Duration::from_secs(2))
        } else {
            command_status_with_timeout("sh", &["-c", "exit 0"], Duration::from_secs(2))
        }
        .expect("command should spawn");

        assert!(
            status
                .expect("command should finish before timeout")
                .success()
        );
    }

    #[test]
    fn command_status_with_timeout_returns_error_for_missing_command() {
        let result = command_status_with_timeout(
            "terman-definitely-missing-command",
            &[],
            Duration::from_secs(2),
        );

        assert!(result.is_err());
    }
}
