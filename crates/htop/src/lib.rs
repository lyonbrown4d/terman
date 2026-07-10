use std::error::Error;

mod app;
mod app_events;
mod app_input;
mod app_terminal;
mod body_layout;
mod cli;
mod core_meter;
mod di;
mod format;
mod footer;
mod help;
mod io_view;
mod interrupt;
mod metrics;
mod meter;
mod model;
mod mouse;
mod mouse_context;
mod mouse_rows;
mod network;
mod network_view;
mod overview_layout;
mod overview_view;
mod process_detail;
mod process_rows;
mod process_status;
mod process_table;
mod processes_view;
mod render;
mod selected_scroll;
mod sort_menu;
mod tab_hitbox;
mod tab_sort;

pub use cli::{HtopArgs, run_with_binary_parse};

pub async fn run(args: HtopArgs) -> Result<(), Box<dyn Error>> {
    di::run(args).await
}
