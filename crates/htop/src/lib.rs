use std::error::Error;

mod app;
mod cli;
mod core_meter;
mod format;
mod metrics;
mod process_detail;
mod render;

pub use cli::{HtopArgs, run_with_binary_parse};

pub async fn run(args: HtopArgs) -> Result<(), Box<dyn Error>> {
    app::run(args).await
}
