use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    hash::{Hash, Hasher},
    io::{self, Write},
    path::PathBuf,
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use sysinfo::{Pid, ProcessesToUpdate, System};

use crate::{ScreenArgs, shell::default_shell};

pub(crate) struct BuiltinScreenSessionGuard {
    path: PathBuf,
}

impl Drop for BuiltinScreenSessionGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct BuiltinScreenSession {
    pub(crate) name: String,
    pub(crate) pid: String,
    pub(crate) cwd: String,
    pub(crate) command: String,
}

pub(crate) fn validate_screen_session_name(name: &str) -> io::Result<()> {
    if name.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "screen 会话名不能为空。",
        ));
    }
    Ok(())
}

pub(crate) fn register_builtin_screen_session(
    args: &ScreenArgs,
) -> io::Result<Option<BuiltinScreenSessionGuard>> {
    let Some(session_name) = &args.session_name else {
        return Ok(None);
    };

    let path = builtin_screen_session_record_path(session_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let cwd = env::current_dir()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|_| String::from("<unknown>"));
    let command = args.command.clone().unwrap_or_else(default_shell);
    let record = BuiltinScreenSession {
        name: session_name.clone(),
        pid: std::process::id().to_string(),
        cwd,
        command,
    };
    let record = serde_json::to_string_pretty(&record)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|err| {
            if err.kind() == io::ErrorKind::AlreadyExists {
                io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    terman_common::builtin_screen_session_exists_hint(session_name),
                )
            } else {
                err
            }
        })?;
    file.write_all(format!("{record}\n").as_bytes())?;

    Ok(Some(BuiltinScreenSessionGuard { path }))
}

pub(crate) fn list_builtin_screen_sessions() -> io::Result<()> {
    let dir = builtin_screen_sessions_dir();
    if !dir.exists() {
        println!("{}", terman_common::builtin_screen_no_sessions_hint());
        return Ok(());
    }

    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);

    let mut sessions = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        let Ok(record) = fs::read_to_string(&path) else {
            continue;
        };
        if let Some(session) = parse_builtin_screen_session_record(&record) {
            if builtin_screen_session_is_alive(&session, &system) {
                sessions.push(session);
            } else {
                let _ = fs::remove_file(path);
            }
        }
    }

    sessions.sort_by(|left, right| left.name.cmp(&right.name));

    if sessions.is_empty() {
        println!("{}", terman_common::builtin_screen_no_sessions_hint());
        return Ok(());
    }

    println!("{}", terman_common::builtin_screen_session_list_header());
    for session in sessions {
        println!(
            "  {}\tpid={}\tcwd={}\tcommand={}",
            session.name, session.pid, session.cwd, session.command
        );
    }

    Ok(())
}

pub(crate) fn builtin_screen_session_is_alive(
    session: &BuiltinScreenSession,
    system: &System,
) -> bool {
    session
        .pid
        .parse::<u32>()
        .ok()
        .map(|pid| system.process(Pid::from_u32(pid)).is_some())
        .unwrap_or(false)
}

pub(crate) fn parse_builtin_screen_session_record(record: &str) -> Option<BuiltinScreenSession> {
    serde_json::from_str(record).ok()
}

fn builtin_screen_session_record_path(name: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    builtin_screen_sessions_dir().join(format!(
        "{}-{:016x}.session",
        sanitize_session_file_name(name),
        hasher.finish()
    ))
}

fn builtin_screen_sessions_dir() -> PathBuf {
    ProjectDirs::from("", "", "terman")
        .map(|dirs| dirs.data_local_dir().join("screen").join("sessions"))
        .unwrap_or_else(|| env::temp_dir().join("terman-screen").join("sessions"))
}

pub(crate) fn sanitize_session_file_name(name: &str) -> String {
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