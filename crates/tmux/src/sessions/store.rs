use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    hash::{Hash, Hasher},
    io::{self, Write},
    path::{Path, PathBuf},
};

use directories::ProjectDirs;

use super::model::{
    AddBuiltinTmuxWindow, BuiltinTmuxSession, KillBuiltinTmuxWindow,
    RenameBuiltinTmuxSession, RenameBuiltinTmuxWindow, parse_builtin_tmux_session_record,
};

pub(crate) fn register_builtin_tmux_session(name: &str) -> io::Result<bool> {
    write_builtin_tmux_session_record(&BuiltinTmuxSession {
        name: name.to_string(),
        windows: 1,
        attached_clients: 0,
        cwd: current_tmux_cwd(),
        command: None,
        pid: None,
        ipc_endpoint: None,
        window_names: vec![String::from("0")],
    })
}

fn current_tmux_cwd() -> String {
    env::current_dir()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|_| String::from("<unknown>"))
}

pub(crate) fn load_builtin_tmux_sessions() -> io::Result<Vec<BuiltinTmuxSession>> {
    let dir = builtin_tmux_sessions_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let Ok(record) = fs::read_to_string(entry.path()) else {
            continue;
        };
        if let Some(session) = parse_builtin_tmux_session_record(&record) {
            sessions.push(session);
        }
    }

    sessions.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(sessions)
}

pub(crate) fn builtin_tmux_session_exists(name: &str) -> io::Result<bool> {
    Ok(load_builtin_tmux_sessions()?
        .into_iter()
        .any(|session| session.name == name))
}

pub(crate) fn add_builtin_tmux_window(name: &str) -> io::Result<AddBuiltinTmuxWindow> {
    let Some((path, mut session)) = find_builtin_tmux_session(name)? else {
        return Ok(AddBuiltinTmuxWindow::SessionMissing);
    };
    ensure_window_names(&mut session);
    let next_index = session.windows;
    session.windows = session.windows.saturating_add(1);
    session.window_names.push(next_index.to_string());
    replace_builtin_tmux_session_record(&path, &session)?;
    Ok(AddBuiltinTmuxWindow::Added(session.windows))
}

pub(crate) fn kill_builtin_tmux_window(name: &str) -> io::Result<KillBuiltinTmuxWindow> {
    let Some((path, mut session)) = find_builtin_tmux_session(name)? else {
        return Ok(KillBuiltinTmuxWindow::SessionMissing);
    };
    ensure_window_names(&mut session);
    if session.windows <= 1 {
        remove_builtin_tmux_session_record(&path)?;
        return Ok(KillBuiltinTmuxWindow::SessionKilled);
    }

    session.windows -= 1;
    let _ = session.window_names.pop();
    replace_builtin_tmux_session_record(&path, &session)?;
    Ok(KillBuiltinTmuxWindow::Killed(session.windows))
}

pub(crate) fn rename_builtin_tmux_window(
    name: &str,
    index: usize,
    new_name: &str,
) -> io::Result<RenameBuiltinTmuxWindow> {
    let Some((path, mut session)) = find_builtin_tmux_session(name)? else {
        return Ok(RenameBuiltinTmuxWindow::SessionMissing);
    };
    ensure_window_names(&mut session);
    let Some(window_name) = session.window_names.get_mut(index) else {
        return Ok(RenameBuiltinTmuxWindow::WindowMissing);
    };

    *window_name = new_name.to_string();
    replace_builtin_tmux_session_record(&path, &session)?;
    Ok(RenameBuiltinTmuxWindow::Renamed)
}

pub(crate) fn rename_builtin_tmux_session(
    old_name: &str,
    new_name: &str,
) -> io::Result<RenameBuiltinTmuxSession> {
    if builtin_tmux_session_exists(new_name)? {
        return Ok(RenameBuiltinTmuxSession::DestinationExists);
    }

    let Some((old_path, mut session)) = find_builtin_tmux_session(old_name)? else {
        return Ok(RenameBuiltinTmuxSession::SourceMissing);
    };

    session.name = new_name.to_string();
    if !write_builtin_tmux_session_record(&session)? {
        return Ok(RenameBuiltinTmuxSession::DestinationExists);
    }
    match fs::remove_file(old_path) {
        Ok(()) => Ok(RenameBuiltinTmuxSession::Renamed),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(RenameBuiltinTmuxSession::Renamed),
        Err(err) => Err(err),
    }
}

pub(crate) fn remove_builtin_tmux_session(name: &str) -> io::Result<bool> {
    let dir = builtin_tmux_sessions_dir();
    if !dir.exists() {
        return Ok(false);
    }

    let mut removed = false;
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        let Ok(record) = fs::read_to_string(&path) else {
            continue;
        };
        if parse_builtin_tmux_session_record(&record)
            .map(|session| session.name == name)
            .unwrap_or(false)
        {
            remove_builtin_tmux_session_record(&path)?;
            removed = true;
        }
    }
    Ok(removed)
}

fn find_builtin_tmux_session(name: &str) -> io::Result<Option<(PathBuf, BuiltinTmuxSession)>> {
    let dir = builtin_tmux_sessions_dir();
    if !dir.exists() {
        return Ok(None);
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        let Ok(record) = fs::read_to_string(&path) else {
            continue;
        };
        let Some(session) = parse_builtin_tmux_session_record(&record) else {
            continue;
        };
        if session.name == name {
            return Ok(Some((path, session)));
        }
    }
    Ok(None)
}

fn ensure_window_names(session: &mut BuiltinTmuxSession) {
    if session.windows == 0 {
        session.windows = 1;
    }
    let window_count = session.windows as usize;
    while session.window_names.len() < window_count {
        session.window_names.push(session.window_names.len().to_string());
    }
    session.window_names.truncate(window_count);
}

fn write_builtin_tmux_session_record(session: &BuiltinTmuxSession) -> io::Result<bool> {
    let path = builtin_tmux_session_record_path(&session.name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    match fs::OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(mut file) => {
            file.write_all(format_builtin_tmux_session_record(session)?.as_bytes())?;
            Ok(true)
        }
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => Ok(false),
        Err(err) => Err(err),
    }
}

fn replace_builtin_tmux_session_record(path: &Path, session: &BuiltinTmuxSession) -> io::Result<()> {
    fs::write(path, format_builtin_tmux_session_record(session)?)
}

fn remove_builtin_tmux_session_record(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

fn format_builtin_tmux_session_record(session: &BuiltinTmuxSession) -> io::Result<String> {
    let record = serde_json::to_string_pretty(session)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    Ok(format!("{record}\n"))
}

fn builtin_tmux_session_record_path(name: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    builtin_tmux_sessions_dir().join(format!(
        "{}-{:016x}.session",
        sanitize_session_file_name(name),
        hasher.finish()
    ))
}

fn builtin_tmux_sessions_dir() -> PathBuf {
    ProjectDirs::from("", "", "terman")
        .map(|dirs| dirs.data_local_dir().join("tmux").join("sessions"))
        .unwrap_or_else(|| env::temp_dir().join("terman-tmux").join("sessions"))
}

fn sanitize_session_file_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                ch
            } else {
                '_'
            }
        })
        .collect();

    if sanitized.is_empty() {
        String::from("session")
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::sanitize_session_file_name;

    #[test]
    fn sanitizes_session_file_name() {
        assert_eq!(sanitize_session_file_name("dev/main"), "dev_main");
        assert_eq!(sanitize_session_file_name(""), "session");
    }
}

