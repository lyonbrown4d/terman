use std::error::Error;

use crate::{command::TmuxCommand, sessions::load_builtin_tmux_sessions};

pub(crate) fn try_run_builtin_tmux_command(command: &TmuxCommand) -> Result<bool, Box<dyn Error>> {
    match command {
        TmuxCommand::ListSessions => {
            list_builtin_tmux_sessions()?;
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