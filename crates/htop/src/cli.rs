use std::error::Error;

use clap::Parser;

#[derive(Clone, Debug, Parser)]
#[command(name = "terman-htop", about = terman_common::builtin_htop_cli_about())]
pub struct HtopArgs {
    #[arg(long, default_value_t = 1000)]
    pub refresh_ms: u64,

    #[arg(long)]
    pub once: bool,
}

pub async fn run_with_binary_parse() -> Result<(), Box<dyn Error>> {
    crate::run(HtopArgs::parse()).await
}