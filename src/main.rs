use clap::{Parser, Subcommand};

mod screen;
mod tmux;

#[derive(clap::Parser)]
#[command(name = "terman")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Screen(screen::ScreenArgs),
    Tmux(tmux::TmuxArgs),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Screen(args)) => screen::run(args),
        Some(Commands::Tmux(args)) => tmux::run(args),
        None => screen::run(screen::ScreenArgs::default()),
    }
}
