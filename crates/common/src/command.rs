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

pub fn terminal_env() -> Vec<(String, String)> {
    let mut vars = passthrough_env();
    if !vars.iter().any(|(key, _)| key == "TERM") {
        vars.push((String::from("TERM"), String::from("xterm-256color")));
    }
    vars
}

#[cfg(test)]
mod tests {
    use super::command_status_with_timeout;
    use std::time::Duration;

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
