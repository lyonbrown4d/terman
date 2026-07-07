use std::error::Error;

mod app;
mod app_input;
mod cli;
mod core_meter;
mod format;
mod footer;
mod help;
mod metrics;
mod meter;
mod model;
mod mouse;
mod network;
mod network_view;
mod process_detail;
mod process_status;
mod render;
mod sort_menu;

pub use cli::{HtopArgs, run_with_binary_parse};

pub async fn run(args: HtopArgs) -> Result<(), Box<dyn Error>> {
    app::run(args).await
}
