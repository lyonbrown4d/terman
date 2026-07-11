use super::TmuxWindowRuntimeConfig;
use crate::pane_runtime::TmuxPaneRuntimeConfig;

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