use clap::{Parser, Subcommand};
use terman_screen::ScreenArgs;
use terman_tmux::TmuxArgs;

#[derive(clap::Parser)]
#[command(name = "terman")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Screen(ScreenArgs),
    Tmux(TmuxArgs),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Screen(args)) => terman_screen::run(args),
        Some(Commands::Tmux(args)) => terman_tmux::run(args),
        None => terman_screen::run(ScreenArgs::default()),
    }
}
