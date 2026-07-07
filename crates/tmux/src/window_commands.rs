use std::{error::Error, io};

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

pub(crate) fn list_builtin_tmux_windows(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let Some(session) = load_builtin_tmux_sessions()?
        .into_iter()
        .find(|session| session.name == target)
    else {
        return Err(session_not_found_error(&target));
    };

    for index in 0..session.windows {
        println!(
            "{}",
            terman_common::builtin_tmux_window_list_entry_hint(
                &target,
                index,
                &session.window_name(index),
            )
        );
    }
    Ok(())
}

pub(crate) fn create_builtin_tmux_window(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let session = load_builtin_tmux_sessions()?
        .into_iter()
        .find(|session| session.name == target);

    match add_builtin_tmux_window(&target)? {
        AddBuiltinTmuxWindow::Added(windows) => {
            if let Some(session) = session.as_ref() {
                request_builtin_tmux_windows_update(session, windows);
            }
            println!(
                "{}",
                terman_common::builtin_tmux_window_created_hint(&target, windows)
            );
            Ok(())
        }
        AddBuiltinTmuxWindow::SessionMissing => Err(session_not_found_error(&target)),
    }
}

pub(crate) fn kill_builtin_tmux_window_command(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let session = load_builtin_tmux_sessions()?
        .into_iter()
        .find(|session| session.name == target);

    match kill_builtin_tmux_window(&target)? {
        KillBuiltinTmuxWindow::Killed(windows) => {
            if let Some(session) = session.as_ref() {
                request_builtin_tmux_windows_update(session, windows);
            }
            println!(
                "{}",
                terman_common::builtin_tmux_window_killed_hint(&target, windows)
            );
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
    }
}

pub(crate) fn rename_builtin_tmux_window_command(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let window_index = target_window_index_arg(args).unwrap_or(0);
    let Some(new_name) = rename_window_name_arg(args) else {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_tmux_window_name_required_hint(),
        )));
    };

    match rename_builtin_tmux_window(&target, window_index, &new_name)? {
        RenameBuiltinTmuxWindow::Renamed => Ok(()),
        RenameBuiltinTmuxWindow::SessionMissing => Err(session_not_found_error(&target)),
        RenameBuiltinTmuxWindow::WindowMissing => Err(window_not_found_error(&target, window_index)),
    }
}

pub(crate) fn select_builtin_tmux_window_command(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let window_index = target_window_index_arg(args).unwrap_or(0) as u32;
    let Some(session) = load_builtin_tmux_sessions()?
        .into_iter()
        .find(|session| session.name == target)
    else {
        return Err(session_not_found_error(&target));
    };
    if window_index >= session.windows {
        return Err(window_not_found_error(&target, window_index as usize));
    }
    let endpoint = session_endpoint(&session);
    match request_endpoint_response(&endpoint, TmuxIpcRequest::SelectWindow { index: window_index })? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => Err(Box::new(io::Error::new(io::ErrorKind::Unsupported, reason))),
        response => Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_tmux_unexpected_info_response_hint(&format!("{response:?}")),
        ))),
    }
}
fn request_builtin_tmux_windows_update(session: &BuiltinTmuxSession, windows: u32) {
    let endpoint = session_endpoint(session);
    let _ = request_endpoint_response(&endpoint, TmuxIpcRequest::UpdateWindows { windows });
}

fn session_endpoint(session: &BuiltinTmuxSession) -> TmuxIpcEndpoint {
    session
        .ipc_endpoint
        .as_deref()
        .map(TmuxIpcEndpoint::from_raw_name)
        .unwrap_or_else(|| TmuxIpcEndpoint::for_session(&session.name))
}

fn required_target_session_name_arg(args: &[String]) -> Result<String, Box<dyn Error>> {
    target_session_name_arg(args).ok_or_else(target_required_error)
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

fn window_not_found_error(target: &str, index: usize) -> Box<dyn Error> {
    Box::new(io::Error::new(
        io::ErrorKind::NotFound,
        terman_common::builtin_tmux_window_not_found_hint(target, index),
    ))
}