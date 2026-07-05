use std::{error::Error, io, thread, time::Duration};

use crate::{
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
    sessions::{BuiltinTmuxSession, load_builtin_tmux_sessions, remove_builtin_tmux_session},
};

pub(crate) fn list_builtin_tmux_sessions() -> Result<(), Box<dyn Error>> {
    let sessions = load_builtin_tmux_sessions()?;
    let mut live_sessions = Vec::new();

    for session in sessions {
        match query_session_info_with_retry(&session) {
            Ok(status) => live_sessions.push(status),
            Err(_) => {
                let _ = remove_builtin_tmux_session(&session.name)?;
            }
        }
    }

    if live_sessions.is_empty() {
        println!("{}", terman_common::builtin_tmux_no_sessions_hint());
        return Ok(());
    }

    for session in live_sessions {
        println!(
            "{}",
            terman_common::builtin_tmux_session_list_entry_hint(
                &session.name,
                session.windows,
                session.attached_clients,
            )
        );
    }

    Ok(())
}

fn query_session_info_with_retry(session: &BuiltinTmuxSession) -> io::Result<LiveTmuxSession> {
    let mut last_error = None;
    for _ in 0..5 {
        match query_session_info(session) {
            Ok(status) => return Ok(status),
            Err(err) => {
                last_error = Some(err);
                thread::sleep(Duration::from_millis(20));
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        io::Error::new(io::ErrorKind::TimedOut, "tmux server did not respond")
    }))
}

fn query_session_info(session: &BuiltinTmuxSession) -> io::Result<LiveTmuxSession> {
    let endpoint = session_endpoint(session);
    match request_endpoint_response(&endpoint, TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info {
            session_name,
            windows,
            attached_clients,
            ..
        } => Ok(LiveTmuxSession {
            name: session_name,
            windows,
            attached_clients,
        }),
        TmuxIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, reason))
        }
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unexpected tmux info response: {response:?}"),
        )),
    }
}

fn session_endpoint(session: &BuiltinTmuxSession) -> TmuxIpcEndpoint {
    session
        .ipc_endpoint
        .as_deref()
        .map(TmuxIpcEndpoint::from_raw_name)
        .unwrap_or_else(|| TmuxIpcEndpoint::for_session(&session.name))
}

struct LiveTmuxSession {
    name: String,
    windows: u32,
    attached_clients: u32,
}