use std::io;

use super::{
    control_parse::control_command_payload,
    control_session::send_session_control_request,
};
use crate::{ScreenArgs, ipc::ScreenIpcRequest};

pub(super) fn request_setenv_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let Some((name, value)) = setenv_parts(&payload) else {
        return Err(setenv_required_error());
    };
    validate_env_name(name)?;
    send_session_control_request(
        args,
        ScreenIpcRequest::SetEnv {
            name: name.to_string(),
            value: value.to_string(),
        },
    )
}

pub(super) fn request_unsetenv_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let Some(name) = env_name_only(&payload) else {
        return Err(unsetenv_required_error());
    };
    validate_env_name(name)?;
    send_session_control_request(
        args,
        ScreenIpcRequest::UnsetEnv {
            name: name.to_string(),
        },
    )
}

fn setenv_parts(payload: &str) -> Option<(&str, &str)> {
    let payload = payload.trim_start();
    let key_end = payload.find(char::is_whitespace)?;
    let key = &payload[..key_end];
    let value = payload[key_end..].trim_start();
    (!key.is_empty()).then_some((key, value))
}

fn env_name_only(payload: &str) -> Option<&str> {
    let mut parts = payload.split_whitespace();
    let name = parts.next()?;
    parts.next().is_none().then_some(name)
}

fn validate_env_name(name: &str) -> io::Result<()> {
    if name.is_empty() || name.contains('=') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_env_name_invalid_hint(),
        ));
    }
    Ok(())
}

fn setenv_required_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_setenv_required_hint(),
    )
}

fn unsetenv_required_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_unsetenv_required_hint(),
    )
}

#[cfg(test)]
mod tests {
    use super::{env_name_only, setenv_parts};

    #[test]
    fn parses_setenv_payload() {
        assert_eq!(setenv_parts("EDITOR vim"), Some(("EDITOR", "vim")));
        assert_eq!(setenv_parts("A value with spaces"), Some(("A", "value with spaces")));
        assert_eq!(setenv_parts("EMPTY "), Some(("EMPTY", "")));
        assert_eq!(setenv_parts("EDITOR"), None);
    }

    #[test]
    fn parses_unsetenv_payload() {
        assert_eq!(env_name_only("EDITOR"), Some("EDITOR"));
        assert_eq!(env_name_only("EDITOR extra"), None);
        assert_eq!(env_name_only(" "), None);
    }
}