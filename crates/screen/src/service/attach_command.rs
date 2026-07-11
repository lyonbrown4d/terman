use std::io::{self, Write};

use super::{

    control::request_screen_control_command,
};
use crate::{ScreenArgs, terminal_prompt::read_screen_prompt};

pub(super) fn prompt_attach_command(session_name: &str) -> io::Result<()> {
    let prompt = terman_common::builtin_screen_attach_command_prompt_hint();
    let Some(command) = read_screen_prompt(prompt.as_str())? else {
        return Ok(());
    };
    let command = command
        .trim()
        .strip_prefix(':')
        .unwrap_or(command.trim())
        .trim();
    if command.is_empty() {
        return Ok(());
    }
    let args = ScreenArgs {
        session_name: Some(session_name.to_string()),
        execute: Some(command.to_string()),
        ..ScreenArgs::default()
    };
    if let Err(error) = request_screen_control_command(&args) {
        let mut stdout = io::stdout();
        write!(stdout, "\r\n{error}\r\n")?;
        stdout.flush()?;
    }
    Ok(())
}