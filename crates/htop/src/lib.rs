use std::error::Error;

mod app;
mod app_input;
mod cli;
mod core_meter;
mod di;
mod format;
mod footer;
mod help;
mod metrics;
mod meter;
mod model;
mod mouse;
mod mouse_context;
mod network;
mod network_view;
mod process_detail;
mod process_status;
mod process_table;
mod render;
mod sort_menu;
mod tab_hitbox;

pub use cli::{HtopArgs, run_with_binary_parse};

pub async fn run(args: HtopArgs) -> Result<(), Box<dyn Error>> {
    di::run(args).await
}
