use std::{fs, io};

use super::control_parse::control_command_payload;
use crate::ScreenArgs;

type ControlExecutor = fn(&ScreenArgs, &str, &str, &[String]) -> io::Result<()>;

pub(super) fn request_source_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
    execute: ControlExecutor,
) -> io::Result<()> {
    let path = control_command_payload(inline_payload, extra_args);
    if path.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_source_path_required_hint(),
        ));
    }

    let source = fs::read_to_string(path)?;
    for command_text in source_command_lines(&source) {
        let (command, inline_payload) = split_control_command(command_text);
        execute(args, command, inline_payload, &[])?;
    }
    Ok(())
}

fn source_command_lines(source: &str) -> impl Iterator<Item = &str> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

fn split_control_command(command: &str) -> (&str, &str) {
    let command = command.trim();
    let command_end = command.find(char::is_whitespace).unwrap_or(command.len());
    let verb = &command[..command_end];
    let payload = command[command_end..].trim_start();
    (verb, payload)
}

#[cfg(test)]
mod tests {
    use super::{source_command_lines, split_control_command};

    #[test]
    fn filters_source_command_lines() {
        let lines: Vec<_> = source_command_lines("\n# comment\ninfo\n  stuff echo hi  \n").collect();
        assert_eq!(lines, vec!["info", "stuff echo hi"]);
    }

    #[test]
    fn splits_source_control_command() {
        assert_eq!(split_control_command("info"), ("info", ""));
        assert_eq!(split_control_command("stuff echo hi"), ("stuff", "echo hi"));
    }
}