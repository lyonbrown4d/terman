use std::{
    env,
    ffi::OsString,
    path::PathBuf,
    process::{Command, ExitStatus},
};

use clap::{Args, Parser, Subcommand};
use terman_common;

#[derive(Parser)]
#[command(name = "terman")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Screen(TrailingArgs),
    Tmux(TrailingArgs),
}

#[derive(Args)]
struct TrailingArgs {
    #[arg(trailing_var_arg = true)]
    args: Vec<OsString>,
}

#[derive(Copy, Clone)]
enum Binary {
    Screen,
    Tmux,
}

fn main() {
    let cli = Cli::parse();

    let (binary, args) = match cli.command {
        Some(Commands::Screen(args)) => (Binary::Screen, args.args),
        Some(Commands::Tmux(args)) => (Binary::Tmux, args.args),
        None => (Binary::Screen, Vec::new()),
    };

    match run_binary(binary, &args) {
        Ok(status) => {
            if let Some(code) = status.code() {
                std::process::exit(code);
            }
            eprintln!("{} child process terminated abnormally.", binary_name(binary));
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("启动 {} 失败: {err}", binary_name(binary));
            std::process::exit(1);
        }
    }
}

fn run_binary(binary: Binary, args: &[OsString]) -> Result<ExitStatus, Box<dyn std::error::Error>> {
    let exe_name = binary_executable_name(&binary);
    let command = resolve_executable_path(&exe_name).unwrap_or_else(|| PathBuf::from(&exe_name));

    Ok(Command::new(command).args(args).status()?)
}

fn resolve_executable_path(exe_name: &str) -> Option<PathBuf> {
    let current_exe = env::current_exe().ok()?;
    if let Some(dir) = current_exe.parent() {
        let candidate = dir.join(exe_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    terman_common::which_binary(exe_name).map(PathBuf::from)
}

fn binary_name(binary: Binary) -> &'static str {
    match binary {
        Binary::Screen => "terman-screen",
        Binary::Tmux => "terman-tmux",
    }
}

fn binary_executable_name(binary: &Binary) -> String {
    let base = binary_name(*binary);
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}
