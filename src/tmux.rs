use std::{
    env,
    error::Error,
    io,
    process::{Command, ExitStatus},
};

use clap::Args;
use which::which;

#[derive(Args, Debug)]
pub struct TmuxArgs {
    /// Additional arguments passed directly to system tmux.
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}

pub fn run(args: TmuxArgs) -> Result<(), Box<dyn Error>> {
    if which("tmux").is_err() {
        if cfg!(windows) {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::NotFound,
                "未检测到 tmux。Windows 上请先安装 MSYS2/WSL 的 tmux，或先使用 terman screen。",
            )));
        }

        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            "未检测到 tmux。请先安装 tmux（apt/yum/brew/pacman）。",
        )));
    }

    if cfg!(windows) {
        eprintln!("检测到 tmux，可用，但 Windows 终端行为可能受环境影响。建议在 WSL/Mintty/WSL2 中体验最佳。\n");
    }

    let mut cmd = Command::new("tmux");
    cmd.args(&args.args)
        .envs(get_passthrough_env())
        .env("TERM", env::var("TERM").unwrap_or_else(|_| String::from("xterm-256color")))
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());

    let status: ExitStatus = cmd.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("tmux 退出码: {status}"),
        )))
    }
}

fn get_passthrough_env() -> impl Iterator<Item = (String, String)> {
    ["TERM", "COLORTERM", "LC_ALL", "LANG", "LC_CTYPE", "TERM_PROGRAM", "TERM_PROGRAM_VERSION"]
        .iter()
        .filter_map(|k| env::var(k).ok().map(|v| (k.to_string(), v)))
        .collect::<Vec<_>>()
        .into_iter()
}
