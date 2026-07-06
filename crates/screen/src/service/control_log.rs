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