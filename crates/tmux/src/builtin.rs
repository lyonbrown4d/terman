use std::{error::Error, io};

use crate::{
    attach::attach_builtin_tmux_session,
    args::{
        rename_session_name_arg, rename_window_name_arg, session_name_arg, target_session_arg,
        target_session_name_arg, target_window_index_arg,
    },
    command::TmuxCommand,
    launcher::spawn_detached_tmux_server,
    sessions::{
        AddBuiltinTmuxWindow, KillBuiltinTmuxWindow, RenameBuiltinTmuxSession,
        RenameBuiltinTmuxWindow, add_builtin_tmux_window, builtin_tmux_session_exists,
        kill_builtin_tmux_window, load_builtin_tmux_sessions, register_builtin_tmux_session,
        remove_builtin_tmux_session, rename_builtin_tmux_session, rename_builtin_tmux_window,
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
        TmuxCommand::HasSession => {
            has_builtin_tmux_session(args)?;
            Ok(true)
        }
        TmuxCommand::RenameSession => {
            rename_builtin_tmux_session_command(args)?;
            Ok(true)
        }
        TmuxCommand::NewWindow => {
            create_builtin_tmux_window(args)?;
            Ok(true)
        }
        TmuxCommand::ListWindows => {
            list_builtin_tmux_windows(args)?;
            Ok(true)
        }
        TmuxCommand::KillWindow => {
            kill_builtin_tmux_window_command(args)?;
            Ok(true)
        }
        TmuxCommand::RenameWindow => {
            rename_builtin_tmux_window_command(args)?;
            Ok(true)
        }
        TmuxCommand::AttachSession => {
            attach_builtin_tmux_session(args)?;
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

fn list_builtin_tmux_windows(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let Some(session) = load_builtin_tmux_sessions()?
        .into_iter()
        .find(|session| session.name == target)
    else {
        return Err(session_not_found_error(&target));
    };

    for index in 0..session.windows {
        println!(
            "{}",
            terman_common::builtin_tmux_window_list_entry_hint(
                &target,
                index,
                &session.window_name(index),
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

    if builtin_tmux_session_exists(&name)? {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::AlreadyExists,
            terman_common::builtin_tmux_session_exists_hint(&name),
        )));
    }

    let server_pid = spawn_detached_tmux_server(&name)?;
    if register_builtin_tmux_session(&name, Some(server_pid.to_string()), None)? {
        println!("{}", terman_common::builtin_tmux_session_created_hint(&name));
        Ok(())
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::AlreadyExists,
            terman_common::builtin_tmux_session_exists_hint(&name),
        )))
    }
}

fn create_builtin_tmux_window(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    match add_builtin_tmux_window(&target)? {
        AddBuiltinTmuxWindow::Added(windows) => {
            println!(
                "{}",
                terman_common::builtin_tmux_window_created_hint(&target, windows)
            );
            Ok(())
        }
        AddBuiltinTmuxWindow::SessionMissing => Err(session_not_found_error(&target)),
    }
}

fn kill_builtin_tmux_window_command(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    match kill_builtin_tmux_window(&target)? {
        KillBuiltinTmuxWindow::Killed(windows) => {
            println!(
                "{}",
                terman_common::builtin_tmux_window_killed_hint(&target, windows)
            );
            Ok(())
        }
        KillBuiltinTmuxWindow::SessionKilled => {
            println!("{}", terman_common::builtin_tmux_session_killed_hint(&target));
            Ok(())
        }
        KillBuiltinTmuxWindow::SessionMissing => Err(session_not_found_error(&target)),
    }
}

fn rename_builtin_tmux_window_command(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let window_index = target_window_index_arg(args).unwrap_or(0);
    let Some(new_name) = rename_window_name_arg(args) else {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_tmux_window_name_required_hint(),
        )));
    };

    match rename_builtin_tmux_window(&target, window_index, &new_name)? {
        RenameBuiltinTmuxWindow::Renamed => Ok(()),
        RenameBuiltinTmuxWindow::SessionMissing => Err(session_not_found_error(&target)),
        RenameBuiltinTmuxWindow::WindowMissing => Err(window_not_found_error(&target, window_index)),
    }
}

fn kill_builtin_tmux_session(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_arg(args)?;
    if remove_builtin_tmux_session(&target)? {
        println!("{}", terman_common::builtin_tmux_session_killed_hint(&target));
        Ok(())
    } else {
        Err(session_not_found_error(&target))
    }
}

fn has_builtin_tmux_session(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_arg(args)?;
    if builtin_tmux_session_exists(&target)? {
        Ok(())
    } else {
        Err(session_not_found_error(&target))
    }
}

fn rename_builtin_tmux_session_command(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_arg(args)?;
    let Some(new_name) = rename_session_name_arg(args) else {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_tmux_session_name_required_hint(),
        )));
    };

    match rename_builtin_tmux_session(&target, &new_name)? {
        RenameBuiltinTmuxSession::Renamed => Ok(()),
        RenameBuiltinTmuxSession::SourceMissing => Err(session_not_found_error(&target)),
        RenameBuiltinTmuxSession::DestinationExists => Err(Box::new(io::Error::new(
            io::ErrorKind::AlreadyExists,
            terman_common::builtin_tmux_session_exists_hint(&new_name),
        ))),
    }
}

fn required_target_session_arg(args: &[String]) -> Result<String, Box<dyn Error>> {
    target_session_arg(args).ok_or_else(target_required_error)
}

fn required_target_session_name_arg(args: &[String]) -> Result<String, Box<dyn Error>> {
    target_session_name_arg(args).ok_or_else(target_required_error)
}

fn target_required_error() -> Box<dyn Error> {
    Box::new(io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_tmux_target_required_hint(),
    ))
}

fn session_not_found_error(target: &str) -> Box<dyn Error> {
    Box::new(io::Error::new(
        io::ErrorKind::NotFound,
        terman_common::builtin_tmux_session_not_found_hint(target),
    ))
}

fn window_not_found_error(target: &str, index: usize) -> Box<dyn Error> {
    Box::new(io::Error::new(
        io::ErrorKind::NotFound,
        terman_common::builtin_tmux_window_not_found_hint(target, index),
    ))
}

fn new_session_is_detached(args: &[String], detached: bool) -> bool {
    detached || args.iter().any(|arg| arg == "-d" || arg == "--detached")
}

#[cfg(test)]
mod tests {
    use super::new_session_is_detached;

    #[test]
    fn detects_detached_new_session() {
        assert!(new_session_is_detached(&["new".into(), "-d".into()], false));
        assert!(new_session_is_detached(&["new".into()], true));
        assert!(!new_session_is_detached(&["new".into()], false));
    }
}

