use std::error::Error;

mod app;
mod app_events;
mod app_input;
mod app_terminal;
mod cli;
mod core_meter;
mod di;
mod format;
mod footer;
mod help;
mod io_view;
mod metrics;
mod meter;
mod model;
mod mouse;
mod mouse_context;
mod mouse_rows;
mod network;
mod network_view;
mod process_detail;
mod process_rows;
mod process_status;
mod process_table;
mod render;
mod sort_menu;
mod tab_hitbox;

pub use cli::{HtopArgs, run_with_binary_parse};

pub async fn run(args: HtopArgs) -> Result<(), Box<dyn Error>> {
    di::run(args).await
}
