use std::{error::Error, io};

use crate::{
    attach::attach_builtin_tmux_session,
    capture::capture_builtin_tmux_pane,
    args::{
        rename_session_name_arg, rename_window_name_arg, target_session_arg,
        target_session_name_arg, target_window_index_arg,
    },
    command::TmuxCommand,
    detach_client::detach_builtin_tmux_clients,
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest},
    lifecycle::{
        kill_builtin_tmux_server, kill_builtin_tmux_session_command,
        request_builtin_tmux_session_quit,
    },
    message::display_builtin_tmux_message,
    new_session::create_builtin_tmux_session,
    send_keys::send_builtin_tmux_keys,
    service::request_endpoint_response,
    sessions::{
        AddBuiltinTmuxWindow, BuiltinTmuxSession, KillBuiltinTmuxWindow,
        RenameBuiltinTmuxSession, RenameBuiltinTmuxWindow, add_builtin_tmux_window,
        kill_builtin_tmux_window, load_builtin_tmux_sessions, rename_builtin_tmux_session,
        rename_builtin_tmux_window,
    },
    status::{list_builtin_tmux_sessions, require_live_builtin_tmux_session},
};

pub(crate) fn try_run_builtin_tmux_command(
    command: &TmuxCommand,
    args: &[String],
    detached: bool,
) -> Result<bool, Box<dyn Error>> {
    match command {
        TmuxCommand::ListSessions => {
            list_builtin_tmux_sessions()?;
            Ok(true)
        }
        TmuxCommand::KillSession => {
            kill_builtin_tmux_session_command(args)?;
            Ok(true)
        }
        TmuxCommand::KillServer => {
            kill_builtin_tmux_server()?;
            Ok(true)
        }
        TmuxCommand::HasSession => {
            has_builtin_tmux_session(args)?;
            Ok(true)
        }
        TmuxCommand::RenameSession => {
            rename_builtin_tmux_session_command(args)?;
            Ok(true)
        }
        TmuxCommand::DisplayMessage => {
            display_builtin_tmux_message(args)?;
            Ok(true)
        }
        TmuxCommand::CapturePane => {
            capture_builtin_tmux_pane(args)?;
            Ok(true)
        }
        TmuxCommand::DetachClient => {
            detach_builtin_tmux_clients(args)?;
            Ok(true)
        }
        TmuxCommand::SendKeys => {
            send_builtin_tmux_keys(args)?;
            Ok(true)
        }
        TmuxCommand::NewWindow => {
            create_builtin_tmux_window(args)?;
            Ok(true)
        }
        TmuxCommand::ListWindows => {
            list_builtin_tmux_windows(args)?;
            Ok(true)
        }
        TmuxCommand::KillWindow => {
            kill_builtin_tmux_window_command(args)?;
            Ok(true)
        }
        TmuxCommand::RenameWindow => {
            rename_builtin_tmux_window_command(args)?;
            Ok(true)
        }
        TmuxCommand::AttachSession => {
            attach_builtin_tmux_session(args)?;
            Ok(true)
        }
        TmuxCommand::NewSession => {
            create_builtin_tmux_session(args, !new_session_is_detached(args, detached))?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn list_builtin_tmux_windows(args: &[String]) -> Result<(), Box<dyn Error>> {
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

fn create_builtin_tmux_window(args: &[String]) -> Result<(), Box<dyn Error>> {
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

fn kill_builtin_tmux_window_command(args: &[String]) -> Result<(), Box<dyn Error>> {
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

fn rename_builtin_tmux_window_command(args: &[String]) -> Result<(), Box<dyn Error>> {
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

fn has_builtin_tmux_session(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_arg(args)?;
    require_live_builtin_tmux_session(&target)
}

fn rename_builtin_tmux_session_command(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_arg(args)?;
    let Some(new_name) = rename_session_name_arg(args) else {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_tmux_session_name_required_hint(),
        )));
    };
    let source_session = load_builtin_tmux_sessions()?
        .into_iter()
        .find(|session| session.name == target);

    match rename_builtin_tmux_session(&target, &new_name)? {
        RenameBuiltinTmuxSession::Renamed => {
            if let Some(session) = source_session {
                request_builtin_tmux_session_rename(&session, &new_name);
            }
            Ok(())
        }
        RenameBuiltinTmuxSession::SourceMissing => Err(session_not_found_error(&target)),
        RenameBuiltinTmuxSession::DestinationExists => Err(Box::new(io::Error::new(
            io::ErrorKind::AlreadyExists,
            terman_common::builtin_tmux_session_exists_hint(&new_name),
        ))),
    }
}

fn request_builtin_tmux_session_rename(session: &BuiltinTmuxSession, name: &str) {
    let endpoint = session_endpoint(session);
    let _ = request_endpoint_response(
        &endpoint,
        TmuxIpcRequest::RenameSession {
            name: name.to_string(),
        },
    );
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

fn required_target_session_arg(args: &[String]) -> Result<String, Box<dyn Error>> {
    target_session_arg(args).ok_or_else(target_required_error)
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

fn new_session_is_detached(args: &[String], detached: bool) -> bool {
    detached || args.iter().any(|arg| arg == "-d" || arg == "--detached")
}

#[cfg(test)]
mod tests {
    use super::new_session_is_detached;

    #[test]
    fn detects_detached_new_session() {
        assert!(new_session_is_detached(&["new".into(), "-d".into()], false));
        assert!(new_session_is_detached(&["new".into()], true));
        assert!(!new_session_is_detached(&["new".into()], false));
    }
}
