use std::{error::Error, io};

use crate::{
    command::TmuxCommand,
    sessions::{
        load_builtin_tmux_sessions, register_builtin_tmux_session, remove_builtin_tmux_session,
    },
};

pub(crate) fn try_run_builtin_tmux_command(
    command: &TmuxCommand,
    args: &[String],
    detached: bool,
) -> Result<bool, Box<dyn Error>> {
    match command {
        TmuxCommand::ListSessions => {
            list_builtin_tmux_sessions()?;
            Ok(true)
        }
        TmuxCommand::KillSession => {
            kill_builtin_tmux_session(args)?;
            Ok(true)
        }
        TmuxCommand::NewSession if new_session_is_detached(args, detached) => {
            create_builtin_tmux_session(args)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn list_builtin_tmux_sessions() -> Result<(), Box<dyn Error>> {
    let sessions = load_builtin_tmux_sessions()?;
    if sessions.is_empty() {
        println!("{}", terman_common::builtin_tmux_no_sessions_hint());
        return Ok(());
    }

    for session in sessions {
        println!(
            "{}",
            terman_common::builtin_tmux_session_list_entry_hint(
                &session.name,
                session.windows,
                session.attached_clients,
            )
        );
    }
    Ok(())
}

fn create_builtin_tmux_session(args: &[String]) -> Result<(), Box<dyn Error>> {
    let Some(name) = session_name_arg(args) else {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_tmux_session_name_required_hint(),
        )));
    };

    if register_builtin_tmux_session(&name)? {
        println!("{}", terman_common::builtin_tmux_session_created_hint(&name));
        Ok(())
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::AlreadyExists,
            terman_common::builtin_tmux_session_exists_hint(&name),
        )))
    }
}

fn kill_builtin_tmux_session(args: &[String]) -> Result<(), Box<dyn Error>> {
    let Some(target) = target_session_arg(args) else {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_tmux_target_required_hint(),
        )));
    };

    if remove_builtin_tmux_session(&target)? {
        println!("{}", terman_common::builtin_tmux_session_killed_hint(&target));
        Ok(())
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            terman_common::builtin_tmux_session_not_found_hint(&target),
        )))
    }
}

fn new_session_is_detached(args: &[String], detached: bool) -> bool {
    detached || args.iter().any(|arg| arg == "-d" || arg == "--detached")
}

fn session_name_arg(args: &[String]) -> Option<String> {
    named_arg(args, "-s", "--session-name")
}

fn target_session_arg(args: &[String]) -> Option<String> {
    named_arg(args, "-t", "--target-session")
}

fn named_arg(args: &[String], short: &str, long: &str) -> Option<String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == short || arg == long {
            return iter.next().cloned();
        }
        if let Some(value) = arg.strip_prefix(short).filter(|value| !value.is_empty()) {
            return Some(value.to_string());
        }
        if let Some(value) = arg.strip_prefix(&format!("{long}=")) {
            return Some(value.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{new_session_is_detached, session_name_arg, target_session_arg};

    #[test]
    fn parses_session_name_arg() {
        assert_eq!(
            session_name_arg(&["new".into(), "-s".into(), "dev".into()]),
            Some(String::from("dev"))
        );
        assert_eq!(
            session_name_arg(&["new".into(), "-sdev".into()]),
            Some(String::from("dev"))
        );
        assert_eq!(
            session_name_arg(&["new".into(), "--session-name=dev".into()]),
            Some(String::from("dev"))
        );
    }

    #[test]
    fn parses_target_session_arg() {
        assert_eq!(
            target_session_arg(&["kill-session".into(), "-t".into(), "dev".into()]),
            Some(String::from("dev"))
        );
        assert_eq!(
            target_session_arg(&["kill-session".into(), "-tdev".into()]),
            Some(String::from("dev"))
        );
        assert_eq!(
            target_session_arg(&["kill-session".into(), "--target-session=dev".into()]),
            Some(String::from("dev"))
        );
    }

    #[test]
    fn detects_detached_new_session() {
        assert!(new_session_is_detached(&["new".into(), "-d".into()], false));
        assert!(new_session_is_detached(&["new".into()], true));
        assert!(!new_session_is_detached(&["new".into()], false));
    }
}