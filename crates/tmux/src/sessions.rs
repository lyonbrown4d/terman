use std::{
    env, fs, io,
    path::PathBuf,
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct BuiltinTmuxSession {
    pub(crate) name: String,
    #[serde(default = "default_window_count")]
    pub(crate) windows: u32,
    #[serde(default)]
    pub(crate) attached_clients: u32,
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
            match fs::remove_file(path) {
                Ok(()) => removed = true,
                Err(err) if err.kind() == io::ErrorKind::NotFound => {}
                Err(err) => return Err(err),
            }
        }
    }
    Ok(removed)
}

pub(crate) fn parse_builtin_tmux_session_record(record: &str) -> Option<BuiltinTmuxSession> {
    serde_json::from_str(record).ok()
}

fn builtin_tmux_sessions_dir() -> PathBuf {
    ProjectDirs::from("", "", "terman")
        .map(|dirs| dirs.data_local_dir().join("tmux").join("sessions"))
        .unwrap_or_else(|| env::temp_dir().join("terman-tmux").join("sessions"))
}

fn default_window_count() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::{BuiltinTmuxSession, parse_builtin_tmux_session_record};

    #[test]
    fn parses_tmux_session_record_with_defaults() {
        let session = parse_builtin_tmux_session_record(r#"{"name":"dev"}"#).unwrap();

        assert_eq!(
            session,
            BuiltinTmuxSession {
                name: String::from("dev"),
                windows: 1,
                attached_clients: 0,
            }
        );
    }
}