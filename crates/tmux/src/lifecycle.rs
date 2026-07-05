use std::{error::Error, io};

use crate::{
    args::target_session_arg,
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest},
    service::request_endpoint_response,
    sessions::{BuiltinTmuxSession, load_builtin_tmux_sessions, remove_builtin_tmux_session},
};

pub(crate) fn kill_builtin_tmux_session_command(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_arg(args)?;
    let Some(session) = load_builtin_tmux_sessions()?
        .into_iter()
        .find(|session| session.name == target)
    else {
        return Err(session_not_found_error(&target));
    };

    request_session_quit(&session);
    if remove_builtin_tmux_session(&target)? {
        println!("{}", terman_common::builtin_tmux_session_killed_hint(&target));
        Ok(())
    } else {
        Err(session_not_found_error(&target))
    }
}

pub(crate) fn kill_builtin_tmux_server() -> Result<(), Box<dyn Error>> {
    let sessions = load_builtin_tmux_sessions()?;
    if sessions.is_empty() {
        println!("{}", terman_common::builtin_tmux_no_sessions_hint());
        return Ok(());
    }

    for session in sessions {
        request_session_quit(&session);
        if remove_builtin_tmux_session(&session.name)? {
            println!(
                "{}",
                terman_common::builtin_tmux_session_killed_hint(&session.name)
            );
        }
    }

    Ok(())
}

fn request_session_quit(session: &BuiltinTmuxSession) {
    let endpoint = session_endpoint(session);
    let _ = request_endpoint_response(&endpoint, TmuxIpcRequest::Quit);
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