use std::io;

use super::{
    control_parse::{control_command_payload, decode_stuff_payload},
    control_session::send_session_control_request,
};
use crate::{
    ScreenArgs,
    ipc::ScreenIpcRequest,
};

pub(super) fn request_register_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let Some((name, text)) = register_parts(&payload) else {
        return Err(invalid_register_payload());
    };
    send_session_control_request(
        args,
        ScreenIpcRequest::SetRegister {
            name: name.to_string(),
            bytes: decode_stuff_payload(text),
        },
    )
}

pub(super) fn request_process_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let Some(name) = register_key(&payload) else {
        return Err(invalid_register_payload());
    };
    send_session_control_request(
        args,
        ScreenIpcRequest::PasteRegister {
            name: name.to_string(),
        },
    )
}

fn register_parts(payload: &str) -> Option<(&str, &str)> {
    let payload = payload.trim_start();
    let key_end = payload.find(char::is_whitespace)?;
    let key = &payload[..key_end];
    let text = payload[key_end..].trim_start();
    (!key.is_empty() && !text.is_empty()).then_some((key, text))
}

fn register_key(payload: &str) -> Option<&str> {
    payload.split_whitespace().next()
}

fn invalid_register_payload() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_stuff_required_hint(),
    )
}

#[cfg(test)]
mod tests {
    use super::{register_key, register_parts};

    #[test]
    fn extracts_register_parts_after_key() {
        assert_eq!(register_parts(". hello"), Some((".", "hello")));
        assert_eq!(register_parts("  a   echo hi"), Some(("a", "echo hi")));
        assert_eq!(register_parts("a"), None);
        assert_eq!(register_parts("  "), None);
    }

    #[test]
    fn extracts_process_register_key() {
        assert_eq!(register_key(". extra"), Some("."));
        assert_eq!(register_key("  a"), Some("a"));
        assert_eq!(register_key("  "), None);
    }
}