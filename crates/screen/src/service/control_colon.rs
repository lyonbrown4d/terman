use std::io;

use super::control_parse::control_command_payload;
use crate::ScreenArgs;

type ControlExecutor = fn(&ScreenArgs, &str, &str, &[String]) -> io::Result<()>;

pub(super) fn request_colon_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
    execute: ControlExecutor,
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let payload = normalize_colon_payload(&payload);
    if payload.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_command_required_hint(),
        ));
    }

    let (command, inline_payload) = split_control_command(payload);
    execute(args, command, inline_payload, &[])
}

fn normalize_colon_payload(payload: &str) -> &str {
    let payload = payload.trim();
    payload.strip_prefix(':').unwrap_or(payload).trim_start()
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
    use super::{normalize_colon_payload, split_control_command};

    #[test]
    fn normalizes_colon_payload() {
        assert_eq!(normalize_colon_payload("info"), "info");
        assert_eq!(normalize_colon_payload(": info"), "info");
        assert_eq!(normalize_colon_payload("  :stuff echo hi"), "stuff echo hi");
    }

    #[test]
    fn splits_nested_control_command() {
        assert_eq!(split_control_command("info"), ("info", ""));
        assert_eq!(split_control_command("stuff echo hi"), ("stuff", "echo hi"));
    }
}