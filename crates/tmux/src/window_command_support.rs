use std::{error::Error, io};

use serde::Serialize;

use crate::{
    args::target_session_name_arg,
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
    sessions::BuiltinTmuxSession,
};

#[derive(Serialize)]
struct TmuxWindowListJson {
    schema_version: u16,
    session: String,
    active_window: u32,
    windows: Vec<TmuxWindowJson>,
}

#[derive(Serialize)]
struct TmuxWindowJson {
    index: u32,
    name: String,
    active: bool,
}

pub(crate) fn list_builtin_tmux_windows_json(session: &BuiltinTmuxSession) -> Result<(), Box<dyn Error>> {
    let TmuxIpcResponse::Info { session_name, active_window, window_indexes, window_names, .. } = request_endpoint_response(&session_endpoint(session), TmuxIpcRequest::Info)? else {
        return Err(unexpected_response_error(TmuxIpcResponse::Rejected { reason: String::from("expected info response") }));
    };
    let windows = window_indexes.into_iter().enumerate().map(|(position, index)| TmuxWindowJson {
        index,
        name: window_names.get(position).cloned().unwrap_or_else(|| index.to_string()),
        active: index == active_window,
    }).collect();
    println!("{}", serde_json::to_string_pretty(&TmuxWindowListJson { schema_version: 1, session: session_name, active_window, windows })?);
    Ok(())
}

pub(crate) fn request_builtin_tmux_window_rename(session: &BuiltinTmuxSession, index: u32, name: String) {
    let _ = request_endpoint_response(&session_endpoint(session), TmuxIpcRequest::RenameWindow { index, name });
}

pub(crate) fn request_builtin_tmux_new_window(
    session: &BuiltinTmuxSession,
    index: u32,
    name: String,
    command: Option<String>,
) {
    let _ = request_endpoint_response(&session_endpoint(session), TmuxIpcRequest::NewWindow { index, name, command });
}

pub(crate) fn request_builtin_tmux_window_kill(session: &BuiltinTmuxSession, index: u32) {
    let _ = request_endpoint_response(&session_endpoint(session), TmuxIpcRequest::KillWindow { index });
}

pub(crate) fn active_window_index(session: &BuiltinTmuxSession) -> Result<u32, Box<dyn Error>> {
    match request_endpoint_response(&session_endpoint(session), TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info { active_window, .. } => Ok(active_window),
        TmuxIpcResponse::Rejected { reason } => Err(Box::new(io::Error::new(io::ErrorKind::Unsupported, reason))),
        response => Err(unexpected_response_error(response)),
    }
}

pub(crate) fn session_endpoint(session: &BuiltinTmuxSession) -> TmuxIpcEndpoint {
    session.ipc_endpoint.as_deref().map(TmuxIpcEndpoint::from_raw_name).unwrap_or_else(|| TmuxIpcEndpoint::for_session(&session.name))
}

pub(crate) fn adjacent_window_index(active_window: u32, indexes: &[u32], forward: bool) -> Option<u32> {
    if indexes.is_empty() { return None; }
    let position = indexes.iter().position(|index| *index == active_window).unwrap_or(0);
    let next = if forward {
        (position + 1) % indexes.len()
    } else if position == 0 {
        indexes.len() - 1
    } else {
        position - 1
    };
    indexes.get(next).copied()
}

pub(crate) fn list_windows_json_requested(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--json")
}

pub(crate) fn required_target_session_name_arg(args: &[String]) -> Result<String, Box<dyn Error>> {
    target_session_name_arg(args).ok_or_else(target_required_error)
}

pub(crate) fn session_not_found_error(target: &str) -> Box<dyn Error> {
    Box::new(io::Error::new(io::ErrorKind::NotFound, terman_common::builtin_tmux_session_not_found_hint(target)))
}

pub(crate) fn window_not_found_error(target: &str, index: usize) -> Box<dyn Error> {
    Box::new(io::Error::new(io::ErrorKind::NotFound, terman_common::builtin_tmux_window_not_found_hint(target, index)))
}

pub(crate) fn unexpected_response_error(response: TmuxIpcResponse) -> Box<dyn Error> {
    Box::new(io::Error::new(io::ErrorKind::InvalidData, terman_common::builtin_tmux_unexpected_info_response_hint(&format!("{response:?}"))))
}

fn target_required_error() -> Box<dyn Error> {
    Box::new(io::Error::new(io::ErrorKind::InvalidInput, terman_common::builtin_tmux_target_required_hint()))
}