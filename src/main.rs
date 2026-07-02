use std::ffi::OsString;

use clap::{Args, Parser, Subcommand};
use terman_screen;
use terman_tmux;

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

    if let Err(err) = run_binary(&binary, &args) {
        eprintln!("启动 {} 失败: {err}", binary_name(&binary));
        std::process::exit(1);
    }
}

fn run_binary(binary: &Binary, args: &[OsString]) -> Result<(), Box<dyn std::error::Error>> {
    match binary {
        Binary::Screen => terman_screen::run_with_args(args),
        Binary::Tmux => terman_tmux::run_with_args(args),
    }
}

fn binary_name(binary: &Binary) -> &'static str {
    match binary {
        Binary::Screen => "terman-screen",
        Binary::Tmux => "terman-tmux",
    }
}
