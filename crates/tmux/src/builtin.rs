use std::error::Error;

use crate::command::TmuxCommand;

pub(crate) fn try_run_builtin_tmux_command(command: &TmuxCommand) -> Result<bool, Box<dyn Error>> {
    match command {
        TmuxCommand::ListSessions => {
            println!("{}", terman_common::builtin_tmux_no_sessions_hint());
            Ok(true)
        }
        _ => Ok(false),
    }
}