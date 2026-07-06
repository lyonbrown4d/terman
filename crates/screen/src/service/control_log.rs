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
    let enabled = parse_log_state(inline_payload, extra_args)?;
    send_targeted_session_control_request(args, ScreenIpcRequest::SetLogEnabled { enabled })
}

pub(super) fn request_deflog_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let enabled = parse_log_state(inline_payload, extra_args)?;
    send_session_control_request(
        args,
        ScreenIpcRequest::SetEnv {
            name: String::from("TERMAN_SCREEN_DEFAULT_LOG"),
            value: if enabled { String::from("on") } else { String::from("off") },
        },
    )
}

fn parse_log_state(inline_payload: &str, extra_args: &[String]) -> io::Result<bool> {
    let payload = control_command_payload(inline_payload, extra_args);
    match payload.trim().to_ascii_lowercase().as_str() {
        "on" | "1" | "true" => Ok(true),
        "off" | "0" | "false" => Ok(false),
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
        assert!(parse_log_state("on", &[]).unwrap());
        assert!(!parse_log_state("off", &[]).unwrap());
        assert!(parse_log_state("", &["true".into()]).unwrap());
        assert!(parse_log_state("toggle", &[]).is_err());
    }
}