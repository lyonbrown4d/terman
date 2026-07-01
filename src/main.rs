use clap::Parser;

mod screen;

#[derive(clap::Parser)]
#[command(name = "terman")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    Screen(screen::ScreenArgs),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Screen(args)) => screen::run(args),
        None => screen::run(screen::ScreenArgs::default()),
    }
}
