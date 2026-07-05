use std::{error::Error, io};

mod builtin;
mod cli;
mod sessions;
mod shell;

pub use cli::{ScreenArgs, run_with_binary_parse};
use builtin::run_builtin_screen;
use sessions::{list_builtin_screen_sessions, validate_screen_session_name};

pub fn run(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    if let Some(session_name) = &args.session_name {
        validate_screen_session_name(session_name)?;
    }
    if let Some(Some(session_name)) = &args.resume {
        validate_screen_session_name(session_name)?;
    }
    if let Some(Some(session_name)) = &args.multi_attach {
        validate_screen_session_name(session_name)?;
    }

    if is_builtin_screen_attach_requested(&args) {
        return Err(Box::new(builtin_screen_attach_unsupported_error(&args)));
    }

    if args.list {
        list_builtin_screen_sessions()?;
        return Ok(());
    }

    run_builtin_screen(args)
}

fn is_builtin_screen_attach_requested(args: &ScreenArgs) -> bool {
    args.resume.is_some() || args.multi_attach.is_some()
}

fn builtin_screen_attach_unsupported_error(args: &ScreenArgs) -> io::Error {
    let mode = if args.resume.is_some() {
        "恢复 detached 会话"
    } else {
        "多端附加会话"
    };
    io::Error::new(
        io::ErrorKind::Unsupported,
        format!(
            "内置 screen 暂不支持{mode}。跨平台 attach 需要后续会话服务支持。",
        ),
    )
}
fn screen_failure_message(scope: &str, exit_code: i32, detail: &str) -> String {
    format!("{scope} 失败（退出码 {exit_code}）：{detail}")
}

fn screen_system_runtime_hints(args: &[String], exit_code: i32) -> String {
    let mut hints = Vec::new();

    if is_screen_attach_attempt(args) {
        hints.push(
            "检测到恢复会话参数 (-r/-R/-x)。若会话不存在，先执行 `screen -ls`（或 `terman-screen --system -ls`）确认会话名后重试。".to_string(),
        );
    }

    if is_screen_session_name_arg(args) && exit_code == 1 {
        hints.push(
            "检测到 `-S <name>` 场景，退出码 1 常见于会话名不存在或已有同名会话。先执行 `terman-screen --system -ls`/`screen -ls` 查看后重试。".to_string(),
        );
    }

    let runtime_hint = match exit_code {
        1 => {
            "参数错误、会话名不存在，或参数与 screen 版本不兼容。建议先用 `terman-screen --system --help` 复现最小命令。"
        }
        2 => {
            "通常与权限、终端环境或可执行文件上下文有关。建议在普通终端重试，或先确认 screen 安装和 shell 环境。"
        }
        126 => "无法执行，请确认 screen 可执行文件有执行权限。",
        127 => "未找到本机 screen 可执行文件，请先确认 screen 安装正常且在 PATH。",
        _ => {
            "返回非预期状态，建议先执行 `terman-screen --system --help` 获取可用参数并用最小参数重试。"
        }
    };
    hints.push(runtime_hint.to_string());
    hints.join("\n")
}

fn is_screen_attach_attempt(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "-r" || arg == "-R" || arg == "-x")
}

fn is_screen_session_name_arg(args: &[String]) -> bool {
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        if arg == "-S" {
            return iter.peek().is_some();
        }
    }
    false
}

fn is_screen_detached_arg(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "-d" || arg == "-D" || arg == "--detach")
}

#[cfg(test)]
mod tests {
    use super::{ScreenArgs, is_builtin_screen_attach_requested};
    use super::sessions::{
        BuiltinScreenSession, builtin_screen_session_is_alive, parse_builtin_screen_session_record,
        sanitize_session_file_name,
    };
    use sysinfo::System;

    #[test]
    fn detects_builtin_attach_modes() {
        let resume = ScreenArgs {
            resume: Some(Some(String::from("dev"))),
            ..ScreenArgs::default()
        };
        let multi_attach = ScreenArgs {
            multi_attach: Some(None),
            ..ScreenArgs::default()
        };
        let new_session = ScreenArgs {
            session_name: Some(String::from("dev")),
            ..ScreenArgs::default()
        };

        assert!(is_builtin_screen_attach_requested(&resume));
        assert!(is_builtin_screen_attach_requested(&multi_attach));
        assert!(!is_builtin_screen_attach_requested(&new_session));
    }

    #[test]
    fn sanitizes_builtin_session_record_name() {
        assert_eq!(sanitize_session_file_name("dev/session:1"), "dev_session_1");
    }

    #[test]
    fn parses_builtin_session_record() {
        let record = r#"{"name":"dev","pid":"42","cwd":"C:/repo","command":"pwsh"}"#;
        let parsed = parse_builtin_screen_session_record(record).expect("record should parse");

        assert_eq!(parsed.name, "dev");
        assert_eq!(parsed.pid, "42");
        assert_eq!(parsed.cwd, "C:/repo");
        assert_eq!(parsed.command, "pwsh");
    }

    #[test]
    fn treats_invalid_session_pid_as_dead() {
        let system = System::new();
        let session = BuiltinScreenSession {
            name: String::from("dev"),
            pid: String::from("not-a-pid"),
            cwd: String::from("C:/repo"),
            command: String::from("pwsh"),
        };

        assert!(!builtin_screen_session_is_alive(&session, &system));
    }
}