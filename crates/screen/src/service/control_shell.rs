use std::io;

use super::{
    control_parse::control_command_payload,
    control_session::send_session_control_request,
};
use crate::{ScreenArgs, ipc::ScreenIpcRequest};

pub(super) fn request_shell_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let shell = control_command_payload(inline_payload, extra_args);
    let shell = shell.trim();
    if shell.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_shell_required_hint(),
        ));
    }
    send_session_control_request(
        args,
        ScreenIpcRequest::SetEnv {
            name: String::from("SHELL"),
            value: shell.to_string(),
        },
    )
}
pub(super) fn request_shelltitle_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let title = control_command_payload(inline_payload, extra_args);
    let title = title.trim();
    if title.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_shelltitle_required_hint(),
        ));
    }
    send_session_control_request(
        args,
        ScreenIpcRequest::SetEnv {
            name: String::from("TERMAN_SCREEN_SHELL_TITLE"),
            value: title.to_string(),
        },
    )
}