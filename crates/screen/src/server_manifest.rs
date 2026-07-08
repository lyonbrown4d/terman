use std::{io, sync::{Arc, Mutex}};

use crate::{
    ScreenArgs,
    session_core::ScreenSessionBus,
    sessions::{BuiltinScreenSession, sync_builtin_screen_session_manifest},
    shell::default_shell,
};

pub(crate) fn sync_session_manifest(
    args: &ScreenArgs,
    endpoint_name: &str,
    session_name_state: &Arc<Mutex<String>>,
    bus: &ScreenSessionBus,
) {
    let Ok(session) = server_session_record(args, endpoint_name, session_name_state) else {
        return;
    };
    let status = bus.status_snapshot();
    let _ = sync_builtin_screen_session_manifest(&session, &status);
}

fn server_session_record(
    args: &ScreenArgs,
    endpoint_name: &str,
    session_name_state: &Arc<Mutex<String>>,
) -> io::Result<BuiltinScreenSession> {
    let name = session_name_state
        .lock()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "screen session name lock poisoned"))?
        .clone();
    let cwd = std::env::current_dir()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|_| String::from("<unknown>"));
    Ok(BuiltinScreenSession {
        name,
        pid: std::process::id().to_string(),
        cwd,
        command: args.command.clone().unwrap_or_else(default_shell),
        ipc_endpoint: Some(endpoint_name.to_string()),
    })
}
