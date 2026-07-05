use std::{
    error::Error,
    io,
    process::{Command, ExitStatus, Stdio},
};

use crate::ScreenArgs;

struct ScreenLaunch {
    cmd: String,
}

pub(crate) fn run_system_screen(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    let launch = resolve_screen_launch()?;
    let mut cmd = Command::new(&launch.cmd);
    let system_args = build_system_screen_args(&args);

    let status: ExitStatus = cmd
        .args(&system_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .envs(terman_common::terminal_env())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        let exit_code = status.code().unwrap_or(-1);
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "{}",
                screen_failure_message(
                    "system screen",
                    exit_code,
                    &screen_system_runtime_hints(&system_args, exit_code)
                )
            ),
        )))
    }
}

pub(crate) fn build_system_screen_args(args: &ScreenArgs) -> Vec<String> {
    let mut system_args = Vec::new();

    if args.list {
        system_args.push(String::from("-ls"));
    }

    if args.detach {
        system_args.push(String::from("-d"));
        system_args.push(String::from("-m"));
    }

    if let Some(session_name) = &args.session_name {
        system_args.push(String::from("-S"));
        system_args.push(session_name.clone());
    }

    if let Some(target) = &args.resume {
        system_args.push(String::from("-r"));
        if let Some(target) = target {
            system_args.push(target.clone());
        }
    }

    if let Some(target) = &args.multi_attach {
        system_args.push(String::from("-x"));
        if let Some(target) = target {
            system_args.push(target.clone());
        }
    }

    system_args.extend(args.args.clone());
    system_args
}

pub(crate) fn screen_failure_message(scope: &str, exit_code: i32, detail: &str) -> String {
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

pub(crate) fn is_screen_attach_attempt(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "-r" || arg == "-R" || arg == "-x")
}

pub(crate) fn is_screen_session_name_arg(args: &[String]) -> bool {
    let mut iter = args.iter().peekable();
    while let Some(arg) = iter.next() {
        if arg == "-S" {
            return iter.peek().is_some();
        }
    }
    false
}

pub(crate) fn is_screen_detached_arg(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "-d" || arg == "-D" || arg == "--detach")
}

pub(crate) fn system_screen_fallback_hint() -> &'static str {
    if cfg!(windows) {
        "提示：默认会在 system 失败后回退到内置 screen；如需严格仅用系统 screen，请加 --no-fallback。\n建议先确认本机 screen 可执行文件，或直接使用内置 screen。"
    } else {
        "提示：默认会在 system 失败后回退到内置 screen；如需严格仅用系统 screen，请加 --no-fallback。\n建议先执行：\n  - screen -V\n  - sudo apt/yum/brew install screen\n  - terman-screen --system --no-fallback"
    }
}

fn resolve_screen_launch() -> Result<ScreenLaunch, Box<dyn Error>> {
    if let Some(path) = terman_common::which_binary("screen") {
        return Ok(ScreenLaunch { cmd: path });
    }

    Err(Box::new(io::Error::new(
        io::ErrorKind::NotFound,
        screen_not_found_hint(),
    )))
}

pub(crate) fn screen_not_found_hint() -> String {
    terman_common::native_tool_not_found_hint("screen")
}