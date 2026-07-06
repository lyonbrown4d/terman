use std::io;

use super::{
    control_parse::control_command_payload,
    control_session::{send_session_control_request, send_targeted_session_control_request},
};
use crate::{ScreenArgs, ipc::ScreenIpcRequest};

pub(super) fn request_log_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    match parse_log_state(inline_payload, extra_args)? {
        Some(enabled) => {
            send_targeted_session_control_request(args, ScreenIpcRequest::SetLogEnabled { enabled })
        }
        None => send_targeted_session_control_request(args, ScreenIpcRequest::ToggleLog),
    }
}

pub(super) fn request_deflog_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let Some(enabled) = parse_log_state(inline_payload, extra_args)? else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_log_required_hint(),
        ));
    };
    send_session_control_request(
        args,
        ScreenIpcRequest::SetEnv {
            name: String::from("TERMAN_SCREEN_DEFAULT_LOG"),
            value: if enabled { String::from("on") } else { String::from("off") },
        },
    )
}


pub(super) fn request_logtstamp_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    match parse_logtstamp(inline_payload, extra_args)? {
        LogTimestampCommand::Toggle => {
            send_targeted_session_control_request(args, ScreenIpcRequest::ToggleLogTimestamp)
        }
        LogTimestampCommand::SetEnabled(enabled) => send_targeted_session_control_request(
            args,
            ScreenIpcRequest::SetLogTimestampEnabled { enabled },
        ),
        LogTimestampCommand::After(seconds) => send_targeted_session_control_request(
            args,
            ScreenIpcRequest::SetLogTimestampAfter { seconds },
        ),
        LogTimestampCommand::String(value) => send_targeted_session_control_request(
            args,
            ScreenIpcRequest::SetLogTimestampString { value },
        ),
    }
}

enum LogTimestampCommand {
    Toggle,
    SetEnabled(bool),
    After(u64),
    String(String),
}

fn parse_logtstamp(
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<LogTimestampCommand> {
    let payload = control_command_payload(inline_payload, extra_args);
    let trimmed = payload.trim();
    if trimmed.is_empty() {
        return Ok(LogTimestampCommand::Toggle);
    }
    let lower = trimmed.to_ascii_lowercase();
    match lower.as_str() {
        "on" | "1" | "true" => return Ok(LogTimestampCommand::SetEnabled(true)),
        "off" | "0" | "false" => return Ok(LogTimestampCommand::SetEnabled(false)),
        _ => {}
    }
    if let Some(rest) = trimmed.strip_prefix("after").map(str::trim) {
        let seconds = rest.parse::<u64>().map_err(|_| invalid_logtstamp_payload())?;
        return Ok(LogTimestampCommand::After(seconds));
    }
    if let Some(rest) = trimmed.strip_prefix("string").map(str::trim) {
        if rest.is_empty() {
            return Err(invalid_logtstamp_payload());
        }
        return Ok(LogTimestampCommand::String(rest.to_string()));
    }
    Err(invalid_logtstamp_payload())
}

fn invalid_logtstamp_payload() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_logtstamp_required_hint(),
    )
}
fn parse_log_state(inline_payload: &str, extra_args: &[String]) -> io::Result<Option<bool>> {
    let payload = control_command_payload(inline_payload, extra_args);
    match payload.trim().to_ascii_lowercase().as_str() {
        "" => Ok(None),
        "on" | "1" | "true" => Ok(Some(true)),
        "off" | "0" | "false" => Ok(Some(false)),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_log_required_hint(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_log_state;

    #[test]
    fn parses_log_states() {
        assert_eq!(parse_log_state("", &[]).unwrap(), None);
        assert_eq!(parse_log_state("on", &[]).unwrap(), Some(true));
        assert_eq!(parse_log_state("off", &[]).unwrap(), Some(false));
        assert_eq!(parse_log_state("", &["true".into()]).unwrap(), Some(true));
        assert!(parse_log_state("toggle", &[]).is_err());
    }
}
#[cfg(test)]
mod logtstamp_tests {
    use super::{LogTimestampCommand, parse_logtstamp};

    #[test]
    fn parses_logtstamp_commands() {
        assert!(matches!(parse_logtstamp("", &[]).unwrap(), LogTimestampCommand::Toggle));
        assert!(matches!(parse_logtstamp("on", &[]).unwrap(), LogTimestampCommand::SetEnabled(true)));
        assert!(matches!(parse_logtstamp("after 5", &[]).unwrap(), LogTimestampCommand::After(5)));
        assert!(matches!(parse_logtstamp("string stamp", &[]).unwrap(), LogTimestampCommand::String(value) if value == "stamp"));
        assert!(parse_logtstamp("after nope", &[]).is_err());
    }
}