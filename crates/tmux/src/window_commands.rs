use std::{error::Error, io};

use crate::{
    args::{kill_other_windows_arg, new_window_command_arg, new_window_name_arg, rename_window_name_arg},
    ipc::{TmuxIpcRequest, TmuxIpcResponse},
    lifecycle::request_builtin_tmux_session_quit,
    service::request_endpoint_response,
    sessions::{
        AddBuiltinTmuxWindow, BuiltinTmuxSession, KillBuiltinTmuxWindow, RenameBuiltinTmuxWindow,
        add_builtin_tmux_window_with_name, kill_builtin_tmux_window, load_builtin_tmux_sessions,
        rename_builtin_tmux_window,
    },
    window_command_support::{
        adjacent_window_index, list_builtin_tmux_windows_json,
        list_windows_json_requested, request_builtin_tmux_new_window, resolve_target_window_index,
        request_builtin_tmux_window_kill, request_builtin_tmux_window_rename,
        required_target_session_name_arg, session_endpoint, session_not_found_error,
        unexpected_response_error, window_not_found_error,
    },
};

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
    let window_name = new_window_name_arg(args);
    let command = new_window_command_arg(args);
    match add_builtin_tmux_window_with_name(&target, window_name.as_deref())? {
        AddBuiltinTmuxWindow::Added { windows, index, name } => {
            if let Some(session) = load_builtin_tmux_sessions()?.into_iter().find(|session| session.name == target) {
                request_builtin_tmux_new_window(&session, index, name, command);
            }
            println!("{}", terman_common::builtin_tmux_window_created_hint(&target, windows));
            Ok(())
        }
        AddBuiltinTmuxWindow::SessionMissing => Err(session_not_found_error(&target)),
    }
}

pub(crate) fn kill_builtin_tmux_window_command(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let Some(session) = load_builtin_tmux_sessions()?.into_iter().find(|session| session.name == target) else {
        return Err(session_not_found_error(&target));
    };
    let kill_index = resolve_target_window_index(&session, args)?;
    if kill_other_windows_arg(args) {
        return kill_other_builtin_tmux_windows(&target, &session, kill_index);
    }
    match kill_builtin_tmux_window(&target, Some(kill_index))? {
        KillBuiltinTmuxWindow::Killed { windows, index } => {
            request_builtin_tmux_window_kill(&session, index);
            println!("{}", terman_common::builtin_tmux_window_killed_hint(&target, windows));
            Ok(())
        }
        KillBuiltinTmuxWindow::SessionKilled => {
            request_builtin_tmux_session_quit(&session);
            println!("{}", terman_common::builtin_tmux_session_killed_hint(&target));
            Ok(())
        }
        KillBuiltinTmuxWindow::SessionMissing => Err(session_not_found_error(&target)),
        KillBuiltinTmuxWindow::WindowMissing => Err(window_not_found_error(&target, kill_index)),
    }
}

fn kill_other_builtin_tmux_windows(
    target: &str,
    session: &BuiltinTmuxSession,
    keep_index: u32,
) -> Result<(), Box<dyn Error>> {
    let window_indexes = session.window_indices();
    if !window_indexes.contains(&keep_index) {
        return Err(window_not_found_error(target, keep_index as usize));
    }
    for index in window_indexes.into_iter().filter(|index| *index != keep_index) {
        match kill_builtin_tmux_window(target, Some(index))? {
            KillBuiltinTmuxWindow::Killed { windows, index } => {
                request_builtin_tmux_window_kill(&session, index);
                println!("{}", terman_common::builtin_tmux_window_killed_hint(target, windows));
            }
            KillBuiltinTmuxWindow::SessionMissing => return Err(session_not_found_error(target)),
            KillBuiltinTmuxWindow::WindowMissing => return Err(window_not_found_error(target, index as usize)),
            KillBuiltinTmuxWindow::SessionKilled => return Err(session_not_found_error(target)),
        }
    }
    let _ = request_endpoint_response(&session_endpoint(&session), TmuxIpcRequest::SelectWindow { index: keep_index });
    Ok(())
}

pub(crate) fn rename_builtin_tmux_window_command(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let Some(session) = load_builtin_tmux_sessions()?.into_iter().find(|session| session.name == target) else {
        return Err(session_not_found_error(&target));
    };
    let window_index = resolve_target_window_index(&session, args)? as usize;
    let Some(new_name) = rename_window_name_arg(args) else {
        return Err(Box::new(io::Error::new(io::ErrorKind::InvalidInput, terman_common::builtin_tmux_window_name_required_hint())));
    };
    match rename_builtin_tmux_window(&target, window_index, &new_name)? {
        RenameBuiltinTmuxWindow::Renamed => {
            request_builtin_tmux_window_rename(&session, window_index as u32, new_name);
            Ok(())
        }
        RenameBuiltinTmuxWindow::SessionMissing => Err(session_not_found_error(&target)),
        RenameBuiltinTmuxWindow::WindowMissing => Err(window_not_found_error(&target, window_index)),
    }
}

pub(crate) fn select_builtin_tmux_window_command(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let Some(session) = load_builtin_tmux_sessions()?.into_iter().find(|session| session.name == target) else {
        return Err(session_not_found_error(&target));
    };
    let window_index = resolve_target_window_index(&session, args)?;
    match request_endpoint_response(&session_endpoint(&session), TmuxIpcRequest::SelectWindow { index: window_index })? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => Err(Box::new(io::Error::new(io::ErrorKind::Unsupported, reason))),
        response => Err(unexpected_response_error(response)),
    }
}

pub(crate) fn select_adjacent_builtin_tmux_window_command(
    args: &[String],
    forward: bool,
) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let Some(session) = load_builtin_tmux_sessions()?.into_iter().find(|session| session.name == target) else {
        return Err(session_not_found_error(&target));
    };
    let TmuxIpcResponse::Info { active_window, window_indexes, .. } = request_endpoint_response(
        &session_endpoint(&session),
        TmuxIpcRequest::Info,
    )? else {
        return Err(unexpected_response_error(TmuxIpcResponse::Rejected { reason: String::from("expected info response") }));
    };
    let Some(index) = adjacent_window_index(active_window, &window_indexes, forward) else {
        return Err(window_not_found_error(&target, active_window as usize));
    };
    match request_endpoint_response(&session_endpoint(&session), TmuxIpcRequest::SelectWindow { index })? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => Err(Box::new(io::Error::new(io::ErrorKind::Unsupported, reason))),
        response => Err(unexpected_response_error(response)),
    }
}

pub(crate) fn select_last_builtin_tmux_window_command(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let Some(session) = load_builtin_tmux_sessions()?.into_iter().find(|session| session.name == target) else {
        return Err(session_not_found_error(&target));
    };
    match request_endpoint_response(&session_endpoint(&session), TmuxIpcRequest::LastWindow)? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => Err(Box::new(io::Error::new(io::ErrorKind::Unsupported, reason))),
        response => Err(unexpected_response_error(response)),
    }
}