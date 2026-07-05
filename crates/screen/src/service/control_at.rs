use std::io;

use super::control_parse::control_command_payload;
use crate::ScreenArgs;

type ControlExecutor = fn(&ScreenArgs, &str, &str, &[String]) -> io::Result<()>;

pub(super) fn request_at_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
    execute: ControlExecutor,
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let Some((_, command_text)) = split_at_payload(&payload) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_command_required_hint(),
        ));
    };
    let (command, inline_payload) = split_control_command(command_text);
    execute(args, command, inline_payload, &[])
}

fn split_at_payload(payload: &str) -> Option<(&str, &str)> {
    let payload = payload.trim();
    let target_end = payload.find(char::is_whitespace)?;
    let target = &payload[..target_end];
    let command_text = payload[target_end..].trim_start();
    (!target.is_empty() && !command_text.is_empty()).then_some((target, command_text))
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
    use super::{split_at_payload, split_control_command};

    #[test]
    fn splits_at_payload() {
        assert_eq!(split_at_payload("0 info"), Some(("0", "info")));
        assert_eq!(split_at_payload("# stuff echo hi"), Some(("#", "stuff echo hi")));
        assert_eq!(split_at_payload("0"), None);
        assert_eq!(split_at_payload("  "), None);
    }

    #[test]
    fn splits_nested_control_command() {
        assert_eq!(split_control_command("info"), ("info", ""));
        assert_eq!(split_control_command("stuff echo hi"), ("stuff", "echo hi"));
    }
}