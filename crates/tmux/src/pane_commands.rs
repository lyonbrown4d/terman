use std::{error::Error, io};

use serde::Serialize;

use crate::{
    args::{
        resize_pane_height_arg, resize_pane_width_arg, resize_pane_zoom_arg,
        split_window_command_arg,
        split_window_horizontal_arg, target_pane_index_arg, target_session_name_arg,
        target_window_index_arg,
    },
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
    sessions::{BuiltinTmuxSession, load_builtin_tmux_sessions},
    window_commands::kill_builtin_tmux_window_command,
};

#[derive(Serialize)]
struct TmuxPaneListJson {
    schema_version: u16,
    session: String,
    window_index: u32,
    panes: Vec<TmuxPaneJson>,
}

#[derive(Serialize)]
struct TmuxPaneJson {
    pane_index: u32,
    window_name: String,
    active: bool,
}

struct TmuxPaneInfo {
    window_index: u32,
    window_name: String,
    active_pane: u32,
    pane_indexes: Vec<u32>,
}

pub(crate) fn split_builtin_tmux_pane(args: &[String]) -> Result<(), Box<dyn Error>> {
    let (_, session) = target_session(args)?;
    request_accepted(
        &session,
        TmuxIpcRequest::SplitPane {
            window: target_window_index_arg(args).map(|index| index as u32),
            horizontal: split_window_horizontal_arg(args),
            command: split_window_command_arg(args),
        },
    )
}

pub(crate) fn list_builtin_tmux_panes(args: &[String]) -> Result<(), Box<dyn Error>> {
    let (target, session) = target_session(args)?;
    let info = query_pane_info(
        &session,
        target_window_index_arg(args).map(|index| index as u32),
    )?;
    if list_panes_json_requested(args) {
        return print_panes_json(&target, &info);
    }
    for pane in &info.pane_indexes {
        println!(
            "{}",
            terman_common::builtin_tmux_pane_list_entry_hint(
                &target,
                info.window_index,
                *pane,
                &info.window_name,
                *pane == info.active_pane,
            )
        );
    }
    Ok(())
}

pub(crate) fn display_builtin_tmux_panes(args: &[String]) -> Result<(), Box<dyn Error>> {
    let (target, session) = target_session(args)?;
    let info = query_pane_info(
        &session,
        target_window_index_arg(args).map(|index| index as u32),
    )?;
    let message = info
        .pane_indexes
        .iter()
        .map(|pane| {
            terman_common::builtin_tmux_pane_list_entry_hint(
                &target,
                info.window_index,
                *pane,
                &info.window_name,
                *pane == info.active_pane,
            )
        })
        .collect::<Vec<_>>()
        .join(" | ");
    request_accepted(&session, TmuxIpcRequest::DisplayMessage { message })
}

pub(crate) fn resize_builtin_tmux_pane(args: &[String]) -> Result<(), Box<dyn Error>> {
    let (_, session) = target_session(args)?;
    let zoom = resize_pane_zoom_arg(args);
    let cols = resize_pane_width_arg(args);
    let rows = resize_pane_height_arg(args);
    if !zoom && cols.is_none() && rows.is_none() {
        return Err(pane_size_required_error());
    }
    let info = query_pane_info(
        &session,
        target_window_index_arg(args).map(|index| index as u32),
    )?;
    let pane = target_pane_index_arg(args)
        .map(|index| index as u32)
        .unwrap_or(info.active_pane);
    require_pane(&info, pane)?;
    let request = if zoom {
        TmuxIpcRequest::TogglePaneZoom {
            window: Some(info.window_index),
            pane: Some(pane),
        }
    } else {
        TmuxIpcRequest::ResizePane {
            window: Some(info.window_index),
            pane: Some(pane),
            cols,
            rows,
        }
    };
    request_accepted(&session, request)
}

pub(crate) fn select_builtin_tmux_pane(args: &[String]) -> Result<(), Box<dyn Error>> {
    let (_, session) = target_session(args)?;
    let info = query_pane_info(
        &session,
        target_window_index_arg(args).map(|index| index as u32),
    )?;
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

pub(crate) fn kill_builtin_tmux_pane(args: &[String]) -> Result<(), Box<dyn Error>> {
    let (_, session) = target_session(args)?;
    let info = query_pane_info(
        &session,
        target_window_index_arg(args).map(|index| index as u32),
    )?;
    let pane = target_pane_index_arg(args)
        .map(|index| index as u32)
        .unwrap_or(info.active_pane);
    require_pane(&info, pane)?;
    if info.pane_indexes.len() == 1 {
        return kill_builtin_tmux_window_command(args);
    }
    request_accepted(
        &session,
        TmuxIpcRequest::KillPane {
            window: Some(info.window_index),
            pane: Some(pane),
        },
    )
}

fn target_session(args: &[String]) -> Result<(String, BuiltinTmuxSession), Box<dyn Error>> {
    let target = target_session_name_arg(args).ok_or_else(target_required_error)?;
    let Some(session) = load_builtin_tmux_sessions()?
        .into_iter()
        .find(|session| session.name == target)
    else {
        return Err(session_not_found_error(&target));
    };
    Ok((target, session))
}

fn query_pane_info(
    session: &BuiltinTmuxSession,
    window: Option<u32>,
) -> Result<TmuxPaneInfo, Box<dyn Error>> {
    match request_endpoint_response(&session_endpoint(session), TmuxIpcRequest::PaneInfo { window })?
    {
        TmuxIpcResponse::Panes {
            window_index,
            window_name,
            active_pane,
            pane_indexes,
        } => Ok(TmuxPaneInfo {
            window_index,
            window_name,
            active_pane,
            pane_indexes,
        }),
        TmuxIpcResponse::Rejected { reason } => {
            Err(Box::new(io::Error::new(io::ErrorKind::Unsupported, reason)))
        }
        response => Err(unexpected_response_error(response)),
    }
}

fn request_accepted(
    session: &BuiltinTmuxSession,
    request: TmuxIpcRequest,
) -> Result<(), Box<dyn Error>> {
    match request_endpoint_response(&session_endpoint(session), request)? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => {
            Err(Box::new(io::Error::new(io::ErrorKind::Unsupported, reason)))
        }
        response => Err(unexpected_response_error(response)),
    }
}

fn require_pane(info: &TmuxPaneInfo, pane: u32) -> Result<(), Box<dyn Error>> {
    if info.pane_indexes.contains(&pane) {
        Ok(())
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            terman_common::builtin_tmux_pane_not_found_hint(
                "current",
                info.window_index,
                pane,
            ),
        )))
    }
}

fn print_panes_json(session: &str, info: &TmuxPaneInfo) -> Result<(), Box<dyn Error>> {
    let panes = info
        .pane_indexes
        .iter()
        .map(|pane| TmuxPaneJson {
            pane_index: *pane,
            window_name: info.window_name.clone(),
            active: *pane == info.active_pane,
        })
        .collect();
    println!(
        "{}",
        serde_json::to_string_pretty(&TmuxPaneListJson {
            schema_version: 1,
            session: session.to_string(),
            window_index: info.window_index,
            panes,
        })?
    );
    Ok(())
}

fn session_endpoint(session: &BuiltinTmuxSession) -> TmuxIpcEndpoint {
    session
        .ipc_endpoint
        .as_deref()
        .map(TmuxIpcEndpoint::from_raw_name)
        .unwrap_or_else(|| TmuxIpcEndpoint::for_session(&session.name))
}

fn list_panes_json_requested(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--json")
}

fn pane_size_required_error() -> Box<dyn Error> {
    Box::new(io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_tmux_pane_size_required_hint(),
    ))
}

fn target_required_error() -> Box<dyn Error> {
    Box::new(io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_tmux_target_required_hint(),
    ))
}

fn session_not_found_error(target: &str) -> Box<dyn Error> {
    Box::new(io::Error::new(
        io::ErrorKind::NotFound,
        terman_common::builtin_tmux_session_not_found_hint(target),
    ))
}

fn unexpected_response_error(response: TmuxIpcResponse) -> Box<dyn Error> {
    Box::new(io::Error::new(
        io::ErrorKind::InvalidData,
        terman_common::builtin_tmux_unexpected_info_response_hint(&format!("{response:?}")),
    ))
}
