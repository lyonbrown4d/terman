use std::{
    env,
    error::Error,
    io,
    process::{Command, ExitStatus, Stdio},
};

use clap::Args;
use which::which;

#[derive(Args, Debug)]
pub struct TmuxArgs {
    /// Start tmux detached.
    #[arg(long)]
    pub detached: bool,

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
    let launch = resolve_tmux_launch()?;
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
        passed_args.insert(0, String::from("-d"));
    }

    let status: ExitStatus = cmd
        .args(&passed_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .envs(passthrough_env())
        .env("TERM", env::var("TERM").unwrap_or_else(|_| String::from("xterm-256color")))
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "tmux 退出码: {status}\n{}",
                tmux_launch_failure_hint(&launch),
            ),
        )))
    }
}

fn validate_tmux_launch(launch: &TmuxLaunch) -> Result<(), Box<dyn Error>> {
    let mut probe = Command::new(&launch.cmd);
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

fn tmux_launch_failure_hint(launch: &TmuxLaunch) -> &'static str {
    match launch.kind {
        TmuxKind::Native => tmux_not_found_hint(),
        TmuxKind::Wsl => {
            "当前使用 WSL 回退 tmux 路径，但未检测到可用 tmux。可先在 WSL 内执行 `sudo apt install tmux`（或对应包管理器）进行安装。"
        }
    }
}
fn resolve_tmux_launch() -> Result<TmuxLaunch, Box<dyn Error>> {
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

