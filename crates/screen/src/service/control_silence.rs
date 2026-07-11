use std::io;

use super::{
    control_parse::control_command_payload,
    control_session::send_targeted_session_control_request,
};
use crate::{ScreenArgs, ipc::ScreenIpcRequest};
use crate::session_core::DEFAULT_SILENCE_SECONDS;

pub(super) fn request_silence_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let request = match payload.trim().to_ascii_lowercase().as_str() {
        "" | "toggle" => ScreenIpcRequest::ToggleSilence,
        "on" | "true" => ScreenIpcRequest::SetSilence {
            seconds: Some(DEFAULT_SILENCE_SECONDS),
        },
        "off" | "false" => ScreenIpcRequest::SetSilence { seconds: None },
        value => match value.parse::<u64>() {
            Ok(seconds) if seconds > 0 => ScreenIpcRequest::SetSilence {
                seconds: Some(seconds),
            },
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    terman_common::builtin_screen_control_silence_required_hint(),
                ));
            }
        },
    };
    send_targeted_session_control_request(args, request)
}