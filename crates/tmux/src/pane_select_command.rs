use std::error::Error;

use crate::{
    args::{target_pane_index_arg, target_window_index_arg},
    ipc::TmuxIpcRequest,
    pane_commands::{query_pane_info, request_accepted, require_pane, target_session},
    pane_layout::PaneDirection,
};

pub(crate) fn select_builtin_tmux_pane(args: &[String]) -> Result<(), Box<dyn Error>> {
    let (_, session) = target_session(args)?;
    let info = query_pane_info(
        &session,
        target_window_index_arg(args).map(|index| index as u32),
    )?;
    if let Some(direction) = pane_direction_arg(args) {
        return request_accepted(
            &session,
            TmuxIpcRequest::SelectPaneDirection {
                window: Some(info.window_index),
                direction,
            },
        );
    }
    let pane = target_pane_index_arg(args)
        .map(|index| index as u32)
        .unwrap_or(info.active_pane);
    require_pane(&info, pane)?;
    request_accepted(
        &session,
        TmuxIpcRequest::SelectPane {
            window: Some(info.window_index),
            pane: Some(pane),
        },
    )
}

fn pane_direction_arg(args: &[String]) -> Option<PaneDirection> {
    args.iter().find_map(|arg| match arg.as_str() {
        "-L" => Some(PaneDirection::Left),
        "-R" => Some(PaneDirection::Right),
        "-U" => Some(PaneDirection::Up),
        "-D" => Some(PaneDirection::Down),
        _ => None,
    })
}
