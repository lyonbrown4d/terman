mod runtime;
mod store;

use std::io;

pub(crate) use store::{BuiltinScreenSession, BuiltinScreenSessionGuard};
#[cfg(test)]
pub(crate) use store::{
    builtin_screen_session_is_alive, parse_builtin_screen_session_record, sanitize_session_file_name,
};

use crate::ScreenArgs;

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
) -> io::Result<Option<BuiltinScreenSessionGuard>> {
    store::register_builtin_screen_session(args)
}

pub(crate) fn list_builtin_screen_sessions() -> io::Result<()> {
    let sessions = store::load_alive_builtin_screen_sessions()?;

    if sessions.is_empty() {
        println!("{}", terman_common::builtin_screen_no_sessions_hint());
        return Ok(());
    }

    println!("{}", terman_common::builtin_screen_session_list_header());
    for session in sessions {
        let status = runtime::load_builtin_screen_runtime_status(&session).ok();
        let attach_clients = status
            .as_ref()
            .map(|value| value.attach_clients.to_string())
            .unwrap_or_else(|| String::from("?"));
        let replay_bytes = status
            .as_ref()
            .map(|value| value.replay_bytes.to_string())
            .unwrap_or_else(|| String::from("?"));
        println!(
            "  {}\tpid={}\tattached_clients={}\treplay_bytes={}\tcwd={}\tcommand={}",
            session.name, session.pid, attach_clients, replay_bytes, session.cwd, session.command
        );
    }

    Ok(())
}

pub(crate) fn wipe_builtin_screen_sessions() -> io::Result<()> {
    let removed = store::remove_stale_builtin_screen_session_records()?;
    println!("{}", terman_common::builtin_screen_wipe_complete_hint(removed));
    Ok(())
}

pub(crate) fn find_builtin_screen_session_for_attach(
    target: Option<&str>,
) -> io::Result<BuiltinScreenSession> {
    let sessions = store::load_alive_builtin_screen_sessions()?;
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
