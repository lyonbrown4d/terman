use std::error::Error;

mod app;
mod cli;
mod core_meter;
mod format;
mod footer;
mod help;
mod metrics;
mod meter;
mod model;
mod network;
mod network_view;
mod process_detail;
mod process_status;
mod render;

pub use cli::{HtopArgs, run_with_binary_parse};

pub async fn run(args: HtopArgs) -> Result<(), Box<dyn Error>> {
    app::run(args).await
}
