use std::{error::Error, io};

use crate::{
    attach::attach_builtin_tmux_session,
    buffer_commands::run_builtin_tmux_buffer_command,
    capture::capture_builtin_tmux_pane,
    clients::list_builtin_tmux_clients,
    args::{
        rename_session_name_arg, target_session_arg,
    },
    command::TmuxCommand,
    detach_client::detach_builtin_tmux_clients,
    history::clear_builtin_tmux_history,
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest},
    lifecycle::{
        kill_builtin_tmux_server, kill_builtin_tmux_session_command,
    },
    message::display_builtin_tmux_message,
    new_session::create_builtin_tmux_session,
    pane_commands::{
        display_builtin_tmux_panes, kill_builtin_tmux_pane, list_builtin_tmux_panes,
        resize_builtin_tmux_pane, split_builtin_tmux_pane,
    },
    pane_select_command::select_builtin_tmux_pane,
    pane_swap_command::swap_builtin_tmux_pane,
    refresh_client::refresh_builtin_tmux_client,
    send_keys::{send_builtin_tmux_keys, send_builtin_tmux_prefix},
    service::request_endpoint_response,
    sessions::{
        BuiltinTmuxSession, RenameBuiltinTmuxSession, load_builtin_tmux_sessions,
        rename_builtin_tmux_session,
    },
    status::{list_builtin_tmux_sessions, list_builtin_tmux_sessions_json, require_live_builtin_tmux_session},
    window_options::set_builtin_tmux_window_option,
    window_commands::{create_builtin_tmux_window, kill_builtin_tmux_window_command,
        list_builtin_tmux_windows, rename_builtin_tmux_window_command, select_adjacent_builtin_tmux_window_command, select_builtin_tmux_window_command, select_last_builtin_tmux_window_command},
};

pub(crate) fn try_run_builtin_tmux_command(
    command: &TmuxCommand,
    args: &[String],
    detached: bool,
) -> Result<bool, Box<dyn Error>> {
    match command {
        TmuxCommand::ListSessions => {
            if list_sessions_json_requested(args) {
                list_builtin_tmux_sessions_json()?;
            } else {
                list_builtin_tmux_sessions()?;
            }
            Ok(true)
        }
        TmuxCommand::ListClients => {
            list_builtin_tmux_clients(args)?;
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
        TmuxCommand::DeleteBuffer
        | TmuxCommand::ListBuffers
        | TmuxCommand::PasteBuffer
        | TmuxCommand::SetBuffer
        | TmuxCommand::ShowBuffer => {
            run_builtin_tmux_buffer_command(command, args)?;
            Ok(true)
        }
        TmuxCommand::ClearHistory => {
            clear_builtin_tmux_history(args)?;
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
        TmuxCommand::SendPrefix => {
            send_builtin_tmux_prefix(args)?;
            Ok(true)
        }
        TmuxCommand::SplitWindow => {
            split_builtin_tmux_pane(args)?;
            Ok(true)
        }
        TmuxCommand::SwapPane => {
            swap_builtin_tmux_pane(args)?;
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
        TmuxCommand::ListPanes => {
            list_builtin_tmux_panes(args)?;
            Ok(true)
        }
        TmuxCommand::SelectPane => {
            select_builtin_tmux_pane(args)?;
            Ok(true)
        }
        TmuxCommand::DisplayPanes => {
            display_builtin_tmux_panes(args)?;
            Ok(true)
        }
        TmuxCommand::ResizePane => {
            resize_builtin_tmux_pane(args)?;
            Ok(true)
        }
        TmuxCommand::SetWindowOption => {
            set_builtin_tmux_window_option(args)?;
            Ok(true)
        }
        TmuxCommand::RefreshClient => {
            refresh_builtin_tmux_client(args)?;
            Ok(true)
        }
        TmuxCommand::SelectWindow => {
            select_builtin_tmux_window_command(args)?;
            Ok(true)
        }
        TmuxCommand::NextWindow => {
            select_adjacent_builtin_tmux_window_command(args, true)?;
            Ok(true)
        }
        TmuxCommand::PreviousWindow => {
            select_adjacent_builtin_tmux_window_command(args, false)?;
            Ok(true)
        }
        TmuxCommand::LastWindow => {
            select_last_builtin_tmux_window_command(args)?;
            Ok(true)
        }
        TmuxCommand::KillWindow => {
            kill_builtin_tmux_window_command(args)?;
            Ok(true)
        }
        TmuxCommand::KillPane => {
            kill_builtin_tmux_pane(args)?;
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

fn list_sessions_json_requested(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--json")
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
