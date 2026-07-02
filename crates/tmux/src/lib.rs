use std::{
    env,
    error::Error,
    io,
    process::{Command, ExitStatus, Stdio},
};

use clap::Args;
use which::which;

#[derive(Args, Debug)]
#[command(
    about = "tmux 桥接入口（按原生命令参数透传）",
    after_help = "常见用法示例：\n  - terman tmux new -s dev\n  - terman tmux new-session -s dev\n  - terman tmux attach -t <session>\n  - terman tmux attach-session -t <session>\n  - terman tmux list-sessions\n  - terman tmux --detached new -s dev\n  - terman tmux --detached --wsl new -s dev\n  - terman tmux --wsl new -s dev\n\n排查示例（最小复现）：\n  - 会话不存在：terman tmux attach -t missing-session\n  - 先查看会话：terman tmux list-sessions\n  - 名称冲突：terman tmux new -s demo\n  - 再复现冲突：terman tmux new -s demo\n",
)]
pub struct TmuxArgs {
    /// 等价于 tmux -d，启动会话前台/后台分离。
    /// 已开启 `--detached` 且未显式使用 `new/new-session` 时，tmux 可能按默认行为忽略或返回不同结果。
    #[arg(long)]
    pub detached: bool,

    /// 强制使用 WSL tmux（仅 Windows）。
    #[arg(long)]
    pub wsl: bool,

    /// Directly passed arguments for tmux.
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}

enum TmuxKind {
    Native,
    Wsl,
}

struct TmuxLaunch {
    cmd: String,
    kind: TmuxKind,
    extra_args: Vec<String>,
}

pub fn run(args: TmuxArgs) -> Result<(), Box<dyn Error>> {
    let launch = resolve_tmux_launch(&args)?;
    validate_tmux_launch(&launch)?;

    let mut cmd = Command::new(&launch.cmd);
    match launch.kind {
        TmuxKind::Native => {}
        TmuxKind::Wsl => {
            cmd.args(&launch.extra_args);
            eprintln!("当前使用 WSL tmux 回退路径。建议长期使用 WSL 发行版中的 tmux 以获得更完整行为。");
        }
    }

    let mut passed_args = args.args;
    if args.detached {
        if is_tmux_new_session_command(&passed_args) {
            if !tmux_has_detached_arg(&passed_args) {
                passed_args.insert(0, String::from("-d"));
            }
        } else {
            eprintln!("提示：--detached 通常与 `new/new-session` 配合使用；当前命令将按透传参数原样执行。");
        }
    }

    let status: ExitStatus = cmd
        .args(&passed_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .envs(passthrough_env())
        .env("TERM", env::var("TERM").unwrap_or_else(|_| String::from("xterm-256color")))
        .status()?;

    let exit_code = status.code().unwrap_or(-1);
    if status.success() {
        Ok(())
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "tmux 退出码: {exit_code}\n{}\n{}",
                tmux_launch_failure_hint(&launch),
                tmux_runtime_hints(&passed_args, exit_code, &launch.kind),
            ),
        )))
    }
}

fn validate_tmux_launch(launch: &TmuxLaunch) -> Result<(), Box<dyn Error>> {
    let mut probe = Command::new(&launch.cmd);
    if launch.kind == TmuxKind::Wsl {
        let wsl_check = Command::new(&launch.cmd)
            .arg("-e")
            .arg("which")
            .arg("tmux")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;

        if !wsl_check.success() {
            let code = wsl_check.code().unwrap_or(-1);
            return Err(Box::new(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "WSL tmux 可用性检查失败（退出码 {code}）。当前已进入 WSL 回退路径，但未检测到 WSL 内 tmux。请先安装：wsl -e sudo apt install tmux。{}",
                    tmux_wsl_runtime_hint(),
                ),
            )));
        }
    }

    match launch.kind {
        TmuxKind::Native => {
            probe.arg("-V");
        }
        TmuxKind::Wsl => {
            probe.args(&launch.extra_args);
            probe.arg("-V");
        }
    }

    let status: ExitStatus = probe
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        let code = status.code().unwrap_or(-1);
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("tmux 可用性检查失败（退出码 {code}）。{}", tmux_launch_failure_hint(launch)),
        )))
    }
}

fn tmux_wsl_runtime_hint() -> &'static str {
    "建议先在 WSL 内执行 `wsl -e which tmux` / `wsl -e tmux --version` 确认安装与环境。"
}
fn tmux_launch_failure_hint(launch: &TmuxLaunch) -> &'static str {
    match launch.kind {
        TmuxKind::Native => tmux_not_found_hint(),
        TmuxKind::Wsl => {
            "当前使用 WSL 回退 tmux 路径，但未检测到可用 tmux。可先在 WSL 内执行 `sudo apt install tmux`（或对应包管理器）进行安装。"
        }
    }
}
fn tmux_runtime_hints(args: &[String], exit_code: i32, kind: &TmuxKind) -> String {
    let mut hints = Vec::new();
    if kind == TmuxKind::Wsl && tmux_has_detached_arg(args) {
        hints.push(
            "WSL 回退路径执行 detached 场景失败时，建议先在 WSL 终端直接复现：wsl -e tmux <同样参数>，确认会话名、路径与环境变量无差异。".to_string(),
        );
    }


    if is_tmux_attach_without_target(args) {
        hints.push(
            "你执行了 tmux attach 但未显式指定会话（-t）。建议：terman tmux attach -t <session-name>；或先运行 terman tmux list-sessions 查看可用会话。".to_string(),
        );
    } else if is_tmux_attach_command(args) && exit_code == 1 {
        hints.push(
            "attach 指定了会话但命令返回 1，常见因为目标会话不存在。请先运行 terman tmux list-sessions，确认会话名后重试。".to_string(),
        );
    } else if is_tmux_new_session_command(args) && exit_code == 1 {
        hints.push(
            "新建会话命令返回 1，常见为会话名冲突或会话无法创建。建议先运行 terman tmux list-sessions 确认现有会话，再换名重试。".to_string(),
        );
    }
    if hints.is_empty() {
        let default_hint = match (kind, exit_code) {
            (TmuxKind::Wsl, 1) => {
                "常见失败原因：参数错误、会话不存在，或 WSL 下 tmux 与终端环境不兼容。建议先执行：wsl -e tmux --version 或 wsl -e tmux list-sessions。".to_string()
            }
            (TmuxKind::Wsl, 2) => {
                "在 WSL 回退路径执行失败（退出码 2）：常见为权限/文件系统上下文问题。建议在 WSL 终端直接运行同样参数复现。".to_string()
            }
            (_, 1) => {
                "常见失败原因：参数错误、会话不存在、或 tmux 当前状态不允许该操作。建议确认参数后重试。".to_string()
            }
            (_, 2) => {
                "常见失败原因：tmux 无法执行该命令或权限受限。建议检查可执行文件与文件系统权限。".to_string()
            }
            (_, 126) => {
                "tmux 可执行文件不可执行。可先确认 `tmux` 的权限（chmod +x 或重新安装）。".to_string()
            }
            (_, 127) => match kind {
                TmuxKind::Wsl => {
                    "未在 WSL 中检测到 tmux。请先在 Windows 里执行 `wsl -e which tmux`，或先安装：wsl -e sudo apt install tmux。".to_string()
                }
                TmuxKind::Native => {
                    "未检测到 tmux 命令。请先确认安装路径或在 Windows 下加 --wsl。".to_string()
                }
            },
            (_, 130) => {
                "tmux 被用户中断（Ctrl-C）。如命令应当持久运行可改为后台启动并检查参数。".to_string()
            }
            _ => {
                "建议先用最小参数复现，或结合 `tmux` 原生命令进行排查。".to_string()
            }
        };
        hints.push(default_hint);
    }

    hints.join("\n")
}

fn is_tmux_attach_without_target(args: &[String]) -> bool {
    is_tmux_attach_command(args) && !tmux_attach_has_target(args)
}

fn is_tmux_attach_command(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "attach" || arg == "attach-session")
}

fn tmux_attach_has_target(args: &[String]) -> bool {
    args.iter().any(|arg| {
        arg == "-t"
            || arg == "--target-session"
            || (arg.starts_with("-t") && arg.len() > 2)
            || arg.starts_with("--target-session=")
    })
}

fn is_tmux_new_session_command(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "new" || arg == "new-session")
}

fn tmux_has_detached_arg(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "-d" || arg == "--detached")
}

fn resolve_tmux_launch(args: &TmuxArgs) -> Result<TmuxLaunch, Box<dyn Error>> {
    if args.wsl {
        if !cfg!(windows) {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                "--wsl 仅在 Windows 下可用。",
            )));
        }

        if let Some(path) = which_binary("wsl").or_else(|| which_binary("wsl.exe")) {
            return Ok(TmuxLaunch {
                cmd: path,
                kind: TmuxKind::Wsl,
                extra_args: vec![String::from("-e"), String::from("tmux")],
            });
        }

        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "未检测到 wsl。请先安装或启用 Windows 的 WSL。",
        )));
    }

    if let Some(path) = which_binary("tmux") {
        return Ok(TmuxLaunch {
            cmd: path,
            kind: TmuxKind::Native,
            extra_args: Vec::new(),
        });
    }

    if cfg!(windows) {
        if let Some(path) = which_binary("wsl").or_else(|| which_binary("wsl.exe")) {
            return Ok(TmuxLaunch {
                cmd: path,
                kind: TmuxKind::Wsl,
                extra_args: vec![String::from("-e"), String::from("tmux")],
            });
        }

        return Err(Box::new(io::Error::new(io::ErrorKind::NotFound, tmux_not_found_hint())));
    }

    Err(Box::new(io::Error::new(
        io::ErrorKind::NotFound,
        tmux_not_found_hint(),
    )))
}
fn tmux_not_found_hint() -> &'static str {
    if cfg!(windows) {
        "未检测到 tmux。可选方案：1) 使用 WSL 安装 tmux（推荐）：wsl -e sudo apt install tmux；2) 安装 Windows tmux（如 Scoop 安装）：scoop install tmux；3) 先使用 terman screen 继续工作。"
    } else {
        "未检测到 tmux。请先安装 tmux（apt/yum/brew/pacman）。"
    }
}

fn which_binary(name: &str) -> Option<String> {
    which(name).ok().map(|path| path.to_string_lossy().to_string())
}

fn passthrough_env() -> impl Iterator<Item = (String, String)> {
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
    .collect::<Vec<_>>()
    .into_iter()
}

use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    args: TmuxArgs,
}

pub fn run_with_binary_parse() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    run(cli.args)
}


