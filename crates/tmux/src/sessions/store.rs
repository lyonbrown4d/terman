use std::io;

use crate::ipc::TmuxIpcEndpoint;
use std::path::PathBuf;

use super::model::{
    AddBuiltinTmuxWindow, BuiltinTmuxSession, KillBuiltinTmuxWindow,
    RenameBuiltinTmuxSession, RenameBuiltinTmuxWindow,
};
use super::record::{
    current_tmux_cwd, read_session_record, remove_session_record, replace_session_record,
    session_record_paths, write_new_session_record,
};

pub(crate) fn register_builtin_tmux_session(
    name: &str,
    pid: Option<String>,
    command: Option<String>,
    ipc_endpoint: &TmuxIpcEndpoint,
) -> io::Result<bool> {
    let _ = ipc_endpoint.socket_name()?;

    write_new_session_record(&BuiltinTmuxSession {
        name: name.to_string(),
        windows: 1,
        attached_clients: 0,
        cwd: current_tmux_cwd(),
        command,
        pid,
        ipc_endpoint: Some(ipc_endpoint.raw_name().to_string()),
        window_indexes: vec![0],
        window_names: vec![String::from("0")],
    })
}

pub(crate) fn load_builtin_tmux_sessions() -> io::Result<Vec<BuiltinTmuxSession>> {
    let mut sessions = Vec::new();
    for path in session_record_paths()? {
        if let Some(mut session) = read_session_record(&path) {
            ensure_window_model(&mut session);
            sessions.push(session);
        }
    }

    sessions.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(sessions)
}

pub(crate) fn builtin_tmux_session_exists(name: &str) -> io::Result<bool> {
    Ok(load_builtin_tmux_sessions()?.into_iter().any(|session| session.name == name))
}

pub(crate) fn add_builtin_tmux_window(name: &str) -> io::Result<AddBuiltinTmuxWindow> {
    let Some((path, mut session)) = find_builtin_tmux_session(name)? else {
        return Ok(AddBuiltinTmuxWindow::SessionMissing);
    };
    ensure_window_model(&mut session);
    let next_index = session.window_indexes.iter().copied().max().unwrap_or(0) + 1;
    let window_name = next_index.to_string();
    session.window_indexes.push(next_index);
    session.window_names.push(window_name.clone());
    session.windows = session.window_indexes.len() as u32;
    replace_session_record(&path, &session)?;
    Ok(AddBuiltinTmuxWindow::Added { windows: session.windows, index: next_index, name: window_name })
}

pub(crate) fn kill_builtin_tmux_window(name: &str, target: Option<u32>) -> io::Result<KillBuiltinTmuxWindow> {
    let Some((path, mut session)) = find_builtin_tmux_session(name)? else {
        return Ok(KillBuiltinTmuxWindow::SessionMissing);
    };
    ensure_window_model(&mut session);
    let index = target.unwrap_or_else(|| *session.window_indexes.last().unwrap_or(&0));
    let Some(position) = session.window_indexes.iter().position(|candidate| *candidate == index) else {
        return Ok(KillBuiltinTmuxWindow::WindowMissing);
    };
    if session.window_indexes.len() <= 1 {
        remove_session_record(&path)?;
        return Ok(KillBuiltinTmuxWindow::SessionKilled);
    }

    session.window_indexes.remove(position);
    if position < session.window_names.len() {
        session.window_names.remove(position);
    }
    session.windows = session.window_indexes.len() as u32;
    replace_session_record(&path, &session)?;
    Ok(KillBuiltinTmuxWindow::Killed { windows: session.windows, index })
}

pub(crate) fn rename_builtin_tmux_window(
    name: &str,
    index: usize,
    new_name: &str,
) -> io::Result<RenameBuiltinTmuxWindow> {
    let Some((path, mut session)) = find_builtin_tmux_session(name)? else {
        return Ok(RenameBuiltinTmuxWindow::SessionMissing);
    };
    ensure_window_model(&mut session);
    let Some(position) = session.window_indexes.iter().position(|candidate| *candidate == index as u32) else {
        return Ok(RenameBuiltinTmuxWindow::WindowMissing);
    };
    let Some(window_name) = session.window_names.get_mut(position) else {
        return Ok(RenameBuiltinTmuxWindow::WindowMissing);
    };

    *window_name = new_name.to_string();
    replace_session_record(&path, &session)?;
    Ok(RenameBuiltinTmuxWindow::Renamed)
}

pub(crate) fn rename_builtin_tmux_session(old_name: &str, new_name: &str) -> io::Result<RenameBuiltinTmuxSession> {
    if builtin_tmux_session_exists(new_name)? {
        return Ok(RenameBuiltinTmuxSession::DestinationExists);
    }

    let Some((old_path, mut session)) = find_builtin_tmux_session(old_name)? else {
        return Ok(RenameBuiltinTmuxSession::SourceMissing);
    };

    session.name = new_name.to_string();
    if !write_new_session_record(&session)? {
        return Ok(RenameBuiltinTmuxSession::DestinationExists);
    }
    remove_session_record(&old_path)?;
    Ok(RenameBuiltinTmuxSession::Renamed)
}

pub(crate) fn remove_builtin_tmux_session(name: &str) -> io::Result<bool> {
    let mut removed = false;
    for path in session_record_paths()? {
        if read_session_record(&path).map(|session| session.name == name).unwrap_or(false) {
            remove_session_record(&path)?;
            removed = true;
        }
    }
    Ok(removed)
}

fn find_builtin_tmux_session(name: &str) -> io::Result<Option<(PathBuf, BuiltinTmuxSession)>> {
    for path in session_record_paths()? {
        let Some(mut session) = read_session_record(&path) else {
            continue;
        };
        ensure_window_model(&mut session);
        if session.name == name {
            return Ok(Some((path, session)));
        }
    }
    Ok(None)
}

fn ensure_window_model(session: &mut BuiltinTmuxSession) {
    let mut indexes = session.window_indices();
    if indexes.is_empty() {
        indexes.push(0);
    }
    session.windows = indexes.len() as u32;
    session.window_indexes = indexes;
    while session.window_names.len() < session.window_indexes.len() {
        let index = session.window_indexes[session.window_names.len()];
        session.window_names.push(index.to_string());
    }
    session.window_names.truncate(session.window_indexes.len());
}