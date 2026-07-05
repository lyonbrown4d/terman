use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    hash::{Hash, Hasher},
    io::{self, Write},
    path::{Path, PathBuf},
};

use directories::ProjectDirs;

use super::model::{BuiltinTmuxSession, parse_builtin_tmux_session_record};

pub(super) fn current_tmux_cwd() -> String {
    env::current_dir()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|_| String::from("<unknown>"))
}

pub(super) fn session_record_paths() -> io::Result<Vec<PathBuf>> {
    let dir = builtin_tmux_sessions_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut paths = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            paths.push(entry.path());
        }
    }
    Ok(paths)
}

pub(super) fn read_session_record(path: &Path) -> Option<BuiltinTmuxSession> {
    fs::read_to_string(path)
        .ok()
        .and_then(|record| parse_builtin_tmux_session_record(&record))
}

pub(super) fn write_new_session_record(session: &BuiltinTmuxSession) -> io::Result<bool> {
    let path = builtin_tmux_session_record_path(&session.name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    match fs::OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(mut file) => {
            file.write_all(format_session_record(session)?.as_bytes())?;
            Ok(true)
        }
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => Ok(false),
        Err(err) => Err(err),
    }
}

pub(super) fn replace_session_record(path: &Path, session: &BuiltinTmuxSession) -> io::Result<()> {
    fs::write(path, format_session_record(session)?)
}

pub(super) fn remove_session_record(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

fn format_session_record(session: &BuiltinTmuxSession) -> io::Result<String> {
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
