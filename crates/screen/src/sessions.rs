mod manifest;
mod paths;
mod runtime;
mod store;

use std::{io, thread, time::Duration};

use serde::Serialize;

pub(crate) use store::{BuiltinScreenSession, BuiltinScreenSessionGuard, RenameBuiltinScreenSession};
#[cfg(test)]
pub(crate) use paths::sanitize_session_file_name;
#[cfg(test)]
pub(crate) use store::{builtin_screen_session_is_alive, parse_builtin_screen_session_record};

use crate::{ScreenArgs, ipc::ScreenIpcEndpoint};

const RUNTIME_STATUS_ATTEMPTS: usize = 8;
const RUNTIME_STATUS_RETRY_DELAY: Duration = Duration::from_millis(25);

#[derive(Serialize)]
struct BuiltinScreenSessionListJson {
    schema_version: u16,
    sessions: Vec<BuiltinScreenSessionJson>,
}

#[derive(Serialize)]
struct BuiltinScreenSessionJson {
    id: String,
    name: String,
    pid: String,
    cwd: String,
    command: String,
    ipc_endpoint: Option<String>,
    replay_bytes: usize,
    attach_clients: usize,
    cols: Option<u16>,
    rows: Option<u16>,
    scrollback_lines: usize,
    active_window: usize,
    windows: Vec<crate::ipc::ScreenWindowInfo>,
}

pub(crate) fn validate_screen_session_name(name: &str) -> io::Result<()> {
    if name.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_session_name_empty_hint(),
        ));
    }
    Ok(())
}

pub(crate) fn register_builtin_screen_session(
    args: &ScreenArgs,
    endpoint: &ScreenIpcEndpoint,
    session_name: Option<std::sync::Arc<std::sync::Mutex<String>>>,
) -> io::Result<Option<BuiltinScreenSessionGuard>> {
    store::register_builtin_screen_session(args, endpoint, session_name)
}

pub(crate) fn rename_builtin_screen_session(
    old_name: &str,
    new_name: &str,
) -> io::Result<RenameBuiltinScreenSession> {
    store::rename_builtin_screen_session(old_name, new_name)
}

pub(crate) fn list_builtin_screen_sessions() -> io::Result<()> {
    let sessions = load_reachable_builtin_screen_sessions()?;

    if sessions.is_empty() {
        println!("{}", terman_common::builtin_screen_no_sessions_hint());
        return Ok(());
    }

    println!("{}", terman_common::builtin_screen_session_list_header());
    for (session, status) in sessions {
        println!(
            "{}",
            terman_common::builtin_screen_session_list_entry_hint(
                &session.name,
                &session.pid,
                status.attach_clients,
                status.replay_bytes,
                status.cols,
                status.rows,
                &session.cwd,
                &session.command,
            )
        );
    }

    Ok(())
}


pub(crate) fn list_builtin_screen_sessions_json() -> io::Result<()> {
    let sessions = load_reachable_builtin_screen_sessions()?
        .into_iter()
        .map(json_session)
        .collect();
    let payload = BuiltinScreenSessionListJson {
        schema_version: 1,
        sessions,
    };
    let json = serde_json::to_string_pretty(&payload)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    println!("{json}");
    Ok(())
}
pub(crate) fn wipe_builtin_screen_sessions() -> io::Result<()> {
    let removed = store::remove_stale_builtin_screen_session_records()?
        + remove_unreachable_builtin_screen_sessions()?;
    println!("{}", terman_common::builtin_screen_wipe_complete_hint(removed));
    Ok(())
}

pub(crate) fn find_builtin_screen_session_for_attach(
    target: Option<&str>,
) -> io::Result<BuiltinScreenSession> {
    let sessions = load_reachable_builtin_screen_sessions()?
        .into_iter()
        .map(|(session, _)| session)
        .collect::<Vec<_>>();
    match target {
        Some(name) => sessions
            .into_iter()
            .find(|session| session.name == name)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    terman_common::builtin_screen_session_not_found_hint(name),
                )
            }),
        None if sessions.len() == 1 => Ok(sessions.into_iter().next().expect("one session")),
        None => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_attach_target_required_hint(),
        )),
    }
}

pub(crate) fn sync_builtin_screen_session_manifest(
    session: &BuiltinScreenSession,
    status: &crate::session_core::ScreenSessionStatus,
) -> io::Result<()> {
    manifest::write_status_builtin_screen_session_manifest(session, status)
}

fn json_session(
    (session, status): (BuiltinScreenSession, runtime::BuiltinScreenSessionRuntimeStatus),
) -> BuiltinScreenSessionJson {
    BuiltinScreenSessionJson {
        id: session
            .ipc_endpoint
            .clone()
            .unwrap_or_else(|| format!("screen:{}", session.name)),
        name: session.name,
        pid: session.pid,
        cwd: session.cwd,
        command: session.command,
        ipc_endpoint: session.ipc_endpoint,
        replay_bytes: status.replay_bytes,
        attach_clients: status.attach_clients,
        cols: status.cols,
        rows: status.rows,
        scrollback_lines: status.scrollback_lines,
        active_window: status.active_window,
        windows: status.windows,
    }
}
fn load_reachable_builtin_screen_sessions(
) -> io::Result<Vec<(BuiltinScreenSession, runtime::BuiltinScreenSessionRuntimeStatus)>> {
    let mut reachable = Vec::new();
    for session in store::load_alive_builtin_screen_sessions()? {
        match load_runtime_status_with_retry(&session) {
            Ok(status) => {
                manifest::write_runtime_builtin_screen_session_manifest(&session, &status)?;
                reachable.push((session, status));
            }
            Err(_) => {
                let _ = store::remove_builtin_screen_session_record(&session.name)?;
            }
        }
    }
    Ok(reachable)
}

fn remove_unreachable_builtin_screen_sessions() -> io::Result<usize> {
    let mut removed = 0;
    for session in store::load_alive_builtin_screen_sessions()? {
        if load_runtime_status_with_retry(&session).is_err()
            && store::remove_builtin_screen_session_record(&session.name)?
        {
            removed += 1;
        }
    }
    Ok(removed)
}

fn load_runtime_status_with_retry(
    session: &BuiltinScreenSession,
) -> io::Result<runtime::BuiltinScreenSessionRuntimeStatus> {
    let mut last_error = None;
    for _ in 0..RUNTIME_STATUS_ATTEMPTS {
        match runtime::load_builtin_screen_runtime_status(session) {
            Ok(status) => return Ok(status),
            Err(err) => {
                last_error = Some(err);
                thread::sleep(RUNTIME_STATUS_RETRY_DELAY);
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        io::Error::new(
            io::ErrorKind::TimedOut,
            terman_common::builtin_screen_service_timeout_hint(),
        )
    }))
}
