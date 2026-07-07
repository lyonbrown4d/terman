use std::{error::Error, io};

use serde::Serialize;

use crate::{
    args::{target_pane_index_arg, target_session_name_arg, target_window_index_arg},
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

pub(crate) fn list_builtin_tmux_panes(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let Some(session) = load_builtin_tmux_sessions()?.into_iter().find(|session| session.name == target) else {
        return Err(session_not_found_error(&target));
    };
    let info = query_tmux_info(&session)?;
    let window_index = target_window_index_arg(args).map(|index| index as u32).unwrap_or(info.active_window);
    let Some(window_name) = info.window_name(window_index) else {
        return Err(window_not_found_error(&target, window_index as usize));
    };
    let active = window_index == info.active_window;
    if list_panes_json_requested(args) {
        return print_panes_json(&target, window_index, window_name, active);
    }
    println!("{}", terman_common::builtin_tmux_pane_list_entry_hint(&target, window_index, 0, &window_name, active));
    Ok(())
}

pub(crate) fn kill_builtin_tmux_pane(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let window_index = target_window_index_arg(args).unwrap_or(0) as u32;
    let pane_index = target_pane_index_arg(args).unwrap_or(0) as u32;
    if pane_index != 0 {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            terman_common::builtin_tmux_pane_not_found_hint(&target, window_index, pane_index),
        )));
    }
    kill_builtin_tmux_window_command(args)
}
struct TmuxPaneInfo {
    active_window: u32,
    window_indexes: Vec<u32>,
    window_names: Vec<String>,
}

impl TmuxPaneInfo {
    fn window_name(&self, index: u32) -> Option<String> {
        self.window_indexes
            .iter()
            .position(|candidate| *candidate == index)
            .map(|position| self.window_names.get(position).cloned().unwrap_or_else(|| index.to_string()))
    }
}

fn query_tmux_info(session: &BuiltinTmuxSession) -> Result<TmuxPaneInfo, Box<dyn Error>> {
    match request_endpoint_response(&session_endpoint(session), TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info { active_window, window_indexes, window_names, .. } => Ok(TmuxPaneInfo { active_window, window_indexes, window_names }),
        TmuxIpcResponse::Rejected { reason } => Err(Box::new(io::Error::new(io::ErrorKind::Unsupported, reason))),
        response => Err(unexpected_response_error(response)),
    }
}

fn print_panes_json(
    session: &str,
    window_index: u32,
    window_name: String,
    active: bool,
) -> Result<(), Box<dyn Error>> {
    let panes = vec![TmuxPaneJson { pane_index: 0, window_name, active }];
    println!("{}", serde_json::to_string_pretty(&TmuxPaneListJson { schema_version: 1, session: session.to_string(), window_index, panes })?);
    Ok(())
}

fn session_endpoint(session: &BuiltinTmuxSession) -> TmuxIpcEndpoint {
    session.ipc_endpoint.as_deref().map(TmuxIpcEndpoint::from_raw_name).unwrap_or_else(|| TmuxIpcEndpoint::for_session(&session.name))
}

fn list_panes_json_requested(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--json")
}

fn required_target_session_name_arg(args: &[String]) -> Result<String, Box<dyn Error>> {
    target_session_name_arg(args).ok_or_else(target_required_error)
}

fn target_required_error() -> Box<dyn Error> {
    Box::new(io::Error::new(io::ErrorKind::InvalidInput, terman_common::builtin_tmux_target_required_hint()))
}

fn session_not_found_error(target: &str) -> Box<dyn Error> {
    Box::new(io::Error::new(io::ErrorKind::NotFound, terman_common::builtin_tmux_session_not_found_hint(target)))
}

fn window_not_found_error(target: &str, index: usize) -> Box<dyn Error> {
    Box::new(io::Error::new(io::ErrorKind::NotFound, terman_common::builtin_tmux_window_not_found_hint(target, index)))
}

fn unexpected_response_error(response: TmuxIpcResponse) -> Box<dyn Error> {
    Box::new(io::Error::new(io::ErrorKind::InvalidData, terman_common::builtin_tmux_unexpected_info_response_hint(&format!("{response:?}"))))
}