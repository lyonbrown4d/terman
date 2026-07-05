use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    hash::{Hash, Hasher},
    io::{self, Write},
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RenameBuiltinTmuxSession {
    Renamed,
    SourceMissing,
    DestinationExists,
}

pub(crate) fn register_builtin_tmux_session(name: &str) -> io::Result<bool> {
    write_builtin_tmux_session_record(&BuiltinTmuxSession {
        name: name.to_string(),
        windows: 1,
        attached_clients: 0,
    })
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

fn write_builtin_tmux_session_record(session: &BuiltinTmuxSession) -> io::Result<bool> {
    let path = builtin_tmux_session_record_path(&session.name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let record = serde_json::to_string_pretty(session)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

    match fs::OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(mut file) => {
            file.write_all(format!("{record}\n").as_bytes())?;
            Ok(true)
        }
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => Ok(false),
        Err(err) => Err(err),
    }
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

fn default_window_count() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::{
        BuiltinTmuxSession, parse_builtin_tmux_session_record, sanitize_session_file_name,
    };

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

    #[test]
    fn sanitizes_session_file_name() {
        assert_eq!(sanitize_session_file_name("dev/main"), "dev_main");
        assert_eq!(sanitize_session_file_name(""), "session");
    }
}
