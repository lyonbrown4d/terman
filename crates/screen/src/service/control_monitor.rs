use std::io;

use super::{
    control_parse::control_command_payload,
    control_session::send_targeted_session_control_request,
};
use crate::{ScreenArgs, ipc::ScreenIpcRequest};

pub(super) fn request_monitor_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let enabled = match payload.trim().to_ascii_lowercase().as_str() {
        "" | "toggle" => None,
        "on" | "1" | "true" => Some(true),
        "off" | "0" | "false" => Some(false),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                terman_common::builtin_screen_control_monitor_required_hint(),
            ));
        }
    };
    send_targeted_session_control_request(args, ScreenIpcRequest::SetMonitor { enabled })
}