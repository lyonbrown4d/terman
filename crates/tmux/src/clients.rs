use std::{error::Error, io};

use crate::{
    args::target_session_name_arg,
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
    sessions::{BuiltinTmuxSession, load_builtin_tmux_sessions},
};

pub(crate) fn list_builtin_tmux_clients(args: &[String]) -> Result<(), Box<dyn Error>> {
    let sessions = load_builtin_tmux_sessions()?;

    if let Some(target) = target_session_name_arg(args) {
        let Some(session) = sessions.iter().find(|session| session.name == target) else {
            return Err(session_not_found_error(&target));
        };
        return print_client_status(session);
    }

    if sessions.is_empty() {
        println!("{}", terman_common::builtin_tmux_no_sessions_hint());
        return Ok(());
    }

    for session in &sessions {
        print_client_status(session)?;
    }

    Ok(())
}

fn print_client_status(session: &BuiltinTmuxSession) -> Result<(), Box<dyn Error>> {
    let (session_name, attached_clients) = query_client_status(session)?;
    println!(
        "{}",
        terman_common::builtin_tmux_client_list_entry_hint(&session_name, attached_clients)
    );
    Ok(())
}

fn query_client_status(session: &BuiltinTmuxSession) -> Result<(String, u32), Box<dyn Error>> {
    match request_endpoint_response(&session_endpoint(session), TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info {
            session_name,
            attached_clients,
            ..
        } => Ok((session_name, attached_clients)),
        TmuxIpcResponse::Rejected { reason } => {
            Err(Box::new(io::Error::new(io::ErrorKind::Unsupported, reason)))
        }
        response => Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")),
        ))),
    }
}

fn session_endpoint(session: &BuiltinTmuxSession) -> TmuxIpcEndpoint {
    session
        .ipc_endpoint
        .as_deref()
        .map(TmuxIpcEndpoint::from_raw_name)
        .unwrap_or_else(|| TmuxIpcEndpoint::for_session(&session.name))
}

fn session_not_found_error(target: &str) -> Box<dyn Error> {
    Box::new(io::Error::new(
        io::ErrorKind::NotFound,
        terman_common::builtin_tmux_session_not_found_hint(target),
    ))
}