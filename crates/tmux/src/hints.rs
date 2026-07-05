pub(super) fn tmux_failure_message(scope: &str, exit_code: i32, detail: &str) -> String {
    format!("{scope} 失败（退出码 {exit_code}）：{detail}")
}

pub(super) fn tmux_launch_failure_hint() -> String {
    terman_common::native_tool_not_found_hint("tmux")
}

pub(super) fn tmux_runtime_hints(args: &[String], exit_code: i32) -> String {
    let mut hints = Vec::new();

    if is_tmux_detached_without_tmux_command(args) {
        hints.push(
            "你仅使用了 --detached/ -d 且未带会话子命令。建议改为 `terman-tmux --detached new -s <name>` 或先确认当前参数。".to_string(),
        );
    }

    if is_tmux_detached_without_new_session(args) {
        hints.push(
            "你使用了 -d/--detached，但未与 new/new-session 组合。当前会话未新增时该参数常被透传下发，建议改用：terman-tmux new -d -s <name> 或 terman-tmux --detached new -s <name>。".to_string(),
        );
    } else if is_tmux_attach_without_target(args) {
        hints.push(
            "你执行了 tmux attach 但未显式指定会话（-t）。建议：terman-tmux attach -t <session-name>；或先运行 terman-tmux list-sessions 查看可用会话。".to_string(),
        );
    } else if is_tmux_list_sessions_command(args) && exit_code == 1 {
        hints.push(
            "你执行 list-sessions 失败，常见为用户权限或 tmux 服务端无法启动。可先执行 `tmux -v` 输出调试信息，或重试 `terman-tmux list-sessions`。".to_string(),
        );
    } else if is_tmux_attach_command(args) && exit_code == 1 {
        hints.push(
            "attach 指定了会话但命令返回 1，常见因为目标会话不存在。请先运行 terman-tmux list-sessions，确认会话名后重试。".to_string(),
        );
    } else if is_tmux_new_session_command(args) && exit_code == 1 {
        hints.push(
            "新建会话命令返回 1，常见为会话名冲突或会话无法创建。建议先运行 terman-tmux list-sessions 确认现有会话，再换名重试。".to_string(),
        );
    }

    if hints.is_empty() {
        hints.push(default_runtime_hint(exit_code));
    }

    hints.join("\n")
}

fn default_runtime_hint(exit_code: i32) -> String {
    match exit_code {
        1 => "常见失败原因：参数错误、会话不存在、或 tmux 当前状态不允许该操作。建议确认参数后重试。",
        2 => "常见失败原因：tmux 无法执行该命令或权限受限。建议检查可执行文件与文件系统权限。",
        126 => "tmux 可执行文件不可执行。可先确认 `tmux` 的权限（chmod +x 或重新安装）。",
        127 => "未检测到本机 tmux 命令。请先确认本机安装路径。",
        130 => "tmux 被用户中断（Ctrl-C）。如命令应当持久运行可改为后台启动并检查参数。",
        _ => "建议先用最小参数复现，或结合 `tmux` 原生命令进行排查。",
    }
    .to_string()
}

pub(super) fn is_tmux_detached_without_tmux_command(args: &[String]) -> bool {
    tmux_has_detached_arg(args) && args.iter().all(|arg| arg == "-d" || arg == "--detached")
}

fn is_tmux_detached_without_new_session(args: &[String]) -> bool {
    tmux_has_detached_arg(args) && !is_tmux_new_session_command(args)
}

fn is_tmux_list_sessions_command(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "list-sessions" || arg == "ls")
}

fn is_tmux_attach_without_target(args: &[String]) -> bool {
    is_tmux_attach_command(args) && !tmux_attach_has_target(args)
}

fn is_tmux_attach_command(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "attach" || arg == "attach-session")
}

fn tmux_attach_has_target(args: &[String]) -> bool {
    args.iter().any(|arg| {
        arg == "-t"
            || arg == "--target-session"
            || (arg.starts_with("-t") && arg.len() > 2)
            || arg.starts_with("--target-session=")
    })
}

pub(super) fn is_tmux_new_session_command(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "new" || arg == "new-session")
}

pub(super) fn tmux_has_detached_arg(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "-d" || arg == "--detached")
}

#[cfg(test)]
mod tests {
    use super::{
        is_tmux_attach_without_target, is_tmux_detached_without_new_session,
        is_tmux_detached_without_tmux_command, is_tmux_list_sessions_command,
        is_tmux_new_session_command, tmux_failure_message, tmux_launch_failure_hint,
    };

    #[test]
    fn detects_tmux_detached_and_attach_flags() {
        let args = vec!["-d".to_string()];
        assert!(is_tmux_detached_without_tmux_command(&args));
        assert!(is_tmux_detached_without_new_session(&args));
        assert!(!is_tmux_detached_without_new_session(&[
            "-d".to_string(),
            "new".to_string()
        ]));
    }

    #[test]
    fn detects_tmux_session_detection() {
        let args = vec!["attach".to_string(), "-t".to_string(), "demo".to_string()];
        assert!(!is_tmux_attach_without_target(&args));
        assert!(is_tmux_attach_without_target(&["attach".to_string()]));
        assert!(is_tmux_list_sessions_command(&["list-sessions".to_string()]));
        assert!(is_tmux_list_sessions_command(&["ls".to_string()]));
        assert!(is_tmux_new_session_command(&["new".to_string()]));
    }

    #[test]
    fn tmux_launch_failure_hint_mentions_native_tool() {
        let hint = tmux_launch_failure_hint();
        assert!(hint.contains("tmux"));
    }

    #[test]
    fn tmux_failure_message_formats_error() {
        let msg = tmux_failure_message("tmux", 1, "命令返回失败");
        assert_eq!(msg, "tmux 失败（退出码 1）：命令返回失败");
    }
}