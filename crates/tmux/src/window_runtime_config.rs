use super::{TmuxWindowRuntime, TmuxWindowRuntimeConfig};
use crate::{
    pane_layout::PaneDirection,
    pane_runtime::TmuxPaneRuntimeConfig,
};

pub(super) fn pane_config(
    config: &TmuxWindowRuntimeConfig,
    pane_index: u32,
    cols: u16,
    rows: u16,
    command: Option<String>,
) -> TmuxPaneRuntimeConfig {
    TmuxPaneRuntimeConfig {
        session_name: config.session_name.clone(),
        window_index: config.index,
        window_name: config.name.clone(),
        pane_index,
        command,
        cols,
        rows,
        login_shell: config.login_shell,
    }
}

pub(super) fn select_pane_direction(
    runtime: &mut TmuxWindowRuntime,
    direction: PaneDirection,
) -> bool {
    let selected = runtime
        .view
        .lock()
        .map(|mut view| view.select_pane_direction(direction))
        .unwrap_or(false);
    if selected {
        runtime.resize_from_view();
        runtime.publish_frame();
    }
    selected
}
