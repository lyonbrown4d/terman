use std::{error::Error, io, thread, time::Duration};

use serde::Serialize;

use crate::{
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
    sessions::{BuiltinTmuxSession, load_builtin_tmux_sessions, remove_builtin_tmux_session},
};

#[derive(Serialize)]
struct TmuxSessionListJson {
    schema_version: u16,
    sessions: Vec<TmuxSessionJson>,
}

#[derive(Serialize)]
struct TmuxSessionJson {
    id: String,
    name: String,
    pid: Option<String>,
    cwd: String,
    command: Option<String>,
    ipc_endpoint: Option<String>,
    windows: u32,
    attached_clients: u32,
    active_window: u32,
    window_indexes: Vec<u32>,
    window_names: Vec<String>,
}

pub(crate) fn list_builtin_tmux_sessions() -> Result<(), Box<dyn Error>> {
    let live_sessions = load_live_builtin_tmux_sessions()?;
    if live_sessions.is_empty() {
        println!("{}", terman_common::builtin_tmux_no_sessions_hint());
        return Ok(());
    }

    for session in live_sessions {
        println!(
            "{}",
            terman_common::builtin_tmux_session_list_entry_hint(
                &session.record.name,
                session.windows,
                session.attached_clients,
            )
        );
    }

    Ok(())
}

pub(crate) fn list_builtin_tmux_sessions_json() -> Result<(), Box<dyn Error>> {
    let sessions = load_live_builtin_tmux_sessions()?
        .into_iter()
        .map(tmux_session_json)
        .collect();
    let payload = TmuxSessionListJson {
        schema_version: 1,
        sessions,
    };
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

pub(crate) fn require_live_builtin_tmux_session(target: &str) -> Result<(), Box<dyn Error>> {
    let Some(session) = load_builtin_tmux_sessions()?
        .into_iter()
        .find(|session| session.name == target)
    else {
        return Err(session_not_found_error(target));
    };

    match query_session_info_with_retry(&session) {
        Ok(_) => Ok(()),
        Err(_) => {
            let _ = remove_builtin_tmux_session(target)?;
            Err(session_not_found_error(target))
        }
    }
}

fn load_live_builtin_tmux_sessions() -> io::Result<Vec<LiveTmuxSession>> {
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

    Ok(live_sessions)
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
        io::Error::new(
            io::ErrorKind::TimedOut,
            terman_common::builtin_tmux_server_not_responding_hint(),
        )
    }))
}

fn query_session_info(session: &BuiltinTmuxSession) -> io::Result<LiveTmuxSession> {
    let endpoint = session_endpoint(session);
    match request_endpoint_response(&endpoint, TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info {
            session_name,
            windows,
            attached_clients,
            active_window,
            window_indexes,
            window_names,
            cwd,
        } => {
            let mut record = session.clone();
            record.name = session_name;
            record.cwd = cwd.clone();
            record.windows = windows;
            record.attached_clients = attached_clients;
            Ok(LiveTmuxSession {
                record,
                windows,
                attached_clients,
                active_window,
                window_indexes,
                window_names,
            })
        }
        TmuxIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, reason))
        }
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_tmux_unexpected_info_response_hint(&format!("{response:?}")),
        )),
    }
}

fn tmux_session_json(session: LiveTmuxSession) -> TmuxSessionJson {
    let window_indexes = session.window_indexes;
    let window_names = session.window_names;
    TmuxSessionJson {
        id: session
            .record
            .ipc_endpoint
            .clone()
            .unwrap_or_else(|| format!("tmux:{}", session.record.name)),
        name: session.record.name,
        pid: session.record.pid,
        cwd: session.record.cwd,
        command: session.record.command,
        ipc_endpoint: session.record.ipc_endpoint,
        windows: session.windows,
        attached_clients: session.attached_clients,
        active_window: session.active_window,
        window_indexes,
        window_names,
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

struct LiveTmuxSession {
    record: BuiltinTmuxSession,
    windows: u32,
    attached_clients: u32,
    active_window: u32,
    window_indexes: Vec<u32>,
    window_names: Vec<String>,
}