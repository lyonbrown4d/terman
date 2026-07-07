use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    hash::{Hash, Hasher},
    io,
    path::PathBuf,
};

use chrono::Utc;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::ipc::ScreenWindowInfo;

use super::{
    runtime::BuiltinScreenSessionRuntimeStatus,
    store::{BuiltinScreenSession, sanitize_session_file_name},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct BuiltinScreenSessionManifest {
    schema_version: u16,
    id: String,
    name: String,
    pid: String,
    cwd: String,
    command: String,
    created_at: String,
    updated_at: String,
    ipc_endpoint: Option<String>,
    active_window: usize,
    attach_clients: usize,
    replay_bytes: usize,
    cols: Option<u16>,
    rows: Option<u16>,
    scrollback_lines: usize,
    windows: Vec<BuiltinScreenWindowManifest>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct BuiltinScreenWindowManifest {
    index: usize,
    title: Option<String>,
    active: bool,
    replay_bytes: usize,
}

pub(crate) fn write_initial_builtin_screen_session_manifest(
    session: &BuiltinScreenSession,
) -> io::Result<()> {
    let now = Utc::now().to_rfc3339();
    write_manifest(&BuiltinScreenSessionManifest {
        schema_version: 1,
        id: session_id(session),
        name: session.name.clone(),
        pid: session.pid.clone(),
        cwd: session.cwd.clone(),
        command: session.command.clone(),
        created_at: now.clone(),
        updated_at: now,
        ipc_endpoint: session.ipc_endpoint.clone(),
        active_window: 0,
        attach_clients: 0,
        replay_bytes: 0,
        cols: None,
        rows: None,
        scrollback_lines: 0,
        windows: vec![BuiltinScreenWindowManifest {
            index: 0,
            title: Some(session.command.clone()),
            active: true,
            replay_bytes: 0,
        }],
    })
}

pub(crate) fn write_runtime_builtin_screen_session_manifest(
    session: &BuiltinScreenSession,
    status: &BuiltinScreenSessionRuntimeStatus,
) -> io::Result<()> {
    let path = builtin_screen_session_manifest_path(&session.name);
    let existing = load_manifest(&path);
    let now = Utc::now().to_rfc3339();
    write_manifest(&BuiltinScreenSessionManifest {
        schema_version: 1,
        id: existing
            .as_ref()
            .map(|manifest| manifest.id.clone())
            .unwrap_or_else(|| session_id(session)),
        name: session.name.clone(),
        pid: session.pid.clone(),
        cwd: session.cwd.clone(),
        command: session.command.clone(),
        created_at: existing
            .as_ref()
            .map(|manifest| manifest.created_at.clone())
            .unwrap_or_else(|| now.clone()),
        updated_at: now,
        ipc_endpoint: session.ipc_endpoint.clone(),
        active_window: status.active_window,
        attach_clients: status.attach_clients,
        replay_bytes: status.replay_bytes,
        cols: status.cols,
        rows: status.rows,
        scrollback_lines: status.scrollback_lines,
        windows: status.windows.iter().map(window_manifest).collect(),
    })
}

pub(crate) fn rename_builtin_screen_session_manifest(old_name: &str, new_name: &str) -> io::Result<()> {
    let old_path = builtin_screen_session_manifest_path(old_name);
    if !old_path.exists() {
        return Ok(());
    }
    let new_path = builtin_screen_session_manifest_path(new_name);
    if let Some(parent) = new_path.parent() {
        fs::create_dir_all(parent)?;
    }
    if new_path.exists() {
        fs::remove_file(&new_path)?;
    }
    fs::rename(old_path, &new_path)?;
    if let Some(mut manifest) = load_manifest(&new_path) {
        manifest.name = new_name.to_string();
        manifest.updated_at = Utc::now().to_rfc3339();
        write_manifest(&manifest)?;
    }
    Ok(())
}

pub(crate) fn remove_builtin_screen_session_manifest(name: &str) -> io::Result<bool> {
    match fs::remove_file(builtin_screen_session_manifest_path(name)) {
        Ok(()) => Ok(true),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err),
    }
}

fn window_manifest(window: &ScreenWindowInfo) -> BuiltinScreenWindowManifest {
    BuiltinScreenWindowManifest {
        index: window.index,
        title: Some(window.title.clone()),
        active: window.active,
        replay_bytes: window.replay_bytes,
    }
}

fn write_manifest(manifest: &BuiltinScreenSessionManifest) -> io::Result<()> {
    let path = builtin_screen_session_manifest_path(&manifest.name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(manifest)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    fs::write(path, format!("{json}\n"))
}

fn load_manifest(path: &PathBuf) -> Option<BuiltinScreenSessionManifest> {
    fs::read_to_string(path)
        .ok()
        .and_then(|json| serde_json::from_str(json.as_str()).ok())
}

fn session_id(session: &BuiltinScreenSession) -> String {
    session
        .ipc_endpoint
        .clone()
        .unwrap_or_else(|| format!("screen:{}", session.name))
}

fn builtin_screen_session_manifest_path(name: &str) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    builtin_screen_manifests_dir().join(format!(
        "{}-{:016x}.manifest.json",
        sanitize_session_file_name(name),
        hasher.finish()
    ))
}

fn builtin_screen_manifests_dir() -> PathBuf {
    ProjectDirs::from("", "", "terman")
        .map(|dirs| dirs.data_local_dir().join("screen").join("manifests"))
        .unwrap_or_else(|| env::temp_dir().join("terman-screen").join("manifests"))
}