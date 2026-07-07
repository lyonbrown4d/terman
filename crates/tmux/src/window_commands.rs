use std::{error::Error, io};

use serde::Serialize;

use crate::{
    args::{rename_window_name_arg, target_session_name_arg, target_window_index_arg},
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    lifecycle::request_builtin_tmux_session_quit,
    service::request_endpoint_response,
    sessions::{
        AddBuiltinTmuxWindow, BuiltinTmuxSession, KillBuiltinTmuxWindow,
        RenameBuiltinTmuxWindow, add_builtin_tmux_window, kill_builtin_tmux_window,
        load_builtin_tmux_sessions, rename_builtin_tmux_window,
    },
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

pub(crate) fn list_builtin_tmux_windows(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let Some(session) = load_builtin_tmux_sessions()?.into_iter().find(|session| session.name == target) else {
        return Err(session_not_found_error(&target));
    };
    if list_windows_json_requested(args) {
        return list_builtin_tmux_windows_json(&session);
    }
    for index in session.window_indices() {
        println!("{}", terman_common::builtin_tmux_window_list_entry_hint(&target, index, &session.window_name(index)));
    }
    Ok(())
}

pub(crate) fn create_builtin_tmux_window(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    match add_builtin_tmux_window(&target)? {
        AddBuiltinTmuxWindow::Added { windows, index, name } => {
            if let Some(session) = load_builtin_tmux_sessions()?.into_iter().find(|session| session.name == target) {
                request_builtin_tmux_new_window(&session, index, name);
            }
            println!("{}", terman_common::builtin_tmux_window_created_hint(&target, windows));
            Ok(())
        }
        AddBuiltinTmuxWindow::SessionMissing => Err(session_not_found_error(&target)),
    }
}

pub(crate) fn kill_builtin_tmux_window_command(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let kill_index = target_window_index_arg(args).map(|index| index as u32);
    let session = load_builtin_tmux_sessions()?.into_iter().find(|session| session.name == target);
    match kill_builtin_tmux_window(&target, kill_index)? {
        KillBuiltinTmuxWindow::Killed { windows, index } => {
            if let Some(session) = session.as_ref() {
                request_builtin_tmux_window_kill(session, index);
            }
            println!("{}", terman_common::builtin_tmux_window_killed_hint(&target, windows));
            Ok(())
        }
        KillBuiltinTmuxWindow::SessionKilled => {
            if let Some(session) = session {
                request_builtin_tmux_session_quit(&session);
            }
            println!("{}", terman_common::builtin_tmux_session_killed_hint(&target));
            Ok(())
        }
        KillBuiltinTmuxWindow::SessionMissing => Err(session_not_found_error(&target)),
        KillBuiltinTmuxWindow::WindowMissing => Err(window_not_found_error(&target, kill_index.unwrap_or(0) as usize)),
    }
}

pub(crate) fn rename_builtin_tmux_window_command(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let window_index = target_window_index_arg(args).unwrap_or(0);
    let Some(new_name) = rename_window_name_arg(args) else {
        return Err(Box::new(io::Error::new(io::ErrorKind::InvalidInput, terman_common::builtin_tmux_window_name_required_hint())));
    };
    match rename_builtin_tmux_window(&target, window_index, &new_name)? {
        RenameBuiltinTmuxWindow::Renamed => {
            if let Some(session) = load_builtin_tmux_sessions()?.into_iter().find(|session| session.name == target) {
                request_builtin_tmux_window_rename(&session, window_index as u32, new_name);
            }
            Ok(())
        }
        RenameBuiltinTmuxWindow::SessionMissing => Err(session_not_found_error(&target)),
        RenameBuiltinTmuxWindow::WindowMissing => Err(window_not_found_error(&target, window_index)),
    }
}

pub(crate) fn select_builtin_tmux_window_command(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let window_index = target_window_index_arg(args).unwrap_or(0) as u32;
    let Some(session) = load_builtin_tmux_sessions()?.into_iter().find(|session| session.name == target) else {
        return Err(session_not_found_error(&target));
    };
    if !session.window_indices().contains(&window_index) {
        return Err(window_not_found_error(&target, window_index as usize));
    }
    match request_endpoint_response(&session_endpoint(&session), TmuxIpcRequest::SelectWindow { index: window_index })? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => Err(Box::new(io::Error::new(io::ErrorKind::Unsupported, reason))),
        response => Err(unexpected_response_error(response)),
    }
}

fn list_builtin_tmux_windows_json(session: &BuiltinTmuxSession) -> Result<(), Box<dyn Error>> {
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

fn request_builtin_tmux_window_rename(session: &BuiltinTmuxSession, index: u32, name: String) {
    let _ = request_endpoint_response(&session_endpoint(session), TmuxIpcRequest::RenameWindow { index, name });
}

fn request_builtin_tmux_new_window(session: &BuiltinTmuxSession, index: u32, name: String) {
    let _ = request_endpoint_response(&session_endpoint(session), TmuxIpcRequest::NewWindow { index, name, command: None });
}

fn request_builtin_tmux_window_kill(session: &BuiltinTmuxSession, index: u32) {
    let _ = request_endpoint_response(&session_endpoint(session), TmuxIpcRequest::KillWindow { index });
}

fn session_endpoint(session: &BuiltinTmuxSession) -> TmuxIpcEndpoint {
    session.ipc_endpoint.as_deref().map(TmuxIpcEndpoint::from_raw_name).unwrap_or_else(|| TmuxIpcEndpoint::for_session(&session.name))
}

fn list_windows_json_requested(args: &[String]) -> bool {
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