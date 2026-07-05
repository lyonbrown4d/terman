use std::{error::Error, io};

use crate::{
    command::TmuxCommand,
    sessions::{load_builtin_tmux_sessions, remove_builtin_tmux_session},
};

pub(crate) fn try_run_builtin_tmux_command(
    command: &TmuxCommand,
    args: &[String],
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

fn target_session_arg(args: &[String]) -> Option<String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "-t" || arg == "--target-session" {
            return iter.next().cloned();
        }
        if let Some(target) = arg.strip_prefix("-t").filter(|value| !value.is_empty()) {
            return Some(target.to_string());
        }
        if let Some(target) = arg.strip_prefix("--target-session=") {
            return Some(target.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::target_session_arg;

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
}