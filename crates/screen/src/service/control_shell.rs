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
    set_default_env(args, "SHELL", shell)
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
    set_default_env(args, "TERMAN_SCREEN_SHELL_TITLE", title)
}

pub(super) fn request_term_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let term = control_command_payload(inline_payload, extra_args);
    let term = term.trim();
    if term.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_term_required_hint(),
        ));
    }
    set_default_env(args, "TERM", term)
}

fn set_default_env(args: &ScreenArgs, name: &str, value: &str) -> io::Result<()> {
    send_session_control_request(
        args,
        ScreenIpcRequest::SetEnv {
            name: name.to_string(),
            value: value.to_string(),
        },
    )
}