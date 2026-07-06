use std::{fs, io};

use encoding_rs::Encoding;

use super::{
    control_buffer_encoding::{decode_buffer_bytes, encode_buffer_bytes},
    control_parse::{control_command_payload, decode_stuff_payload},
    control_session::{send_session_control_request, send_targeted_session_control_request},
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
    let (encoding, payload) = register_encoding_payload(&payload)?;
    let Some((name, text)) = register_parts(payload) else {
        return Err(invalid_register_payload());
    };
    let bytes = encode_buffer_bytes(&decode_stuff_payload(text), encoding);
    send_session_control_request(
        args,
        ScreenIpcRequest::SetRegister {
            name: name.to_string(),
            bytes,
        },
    )
}

pub(super) fn request_readreg_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let (encoding, payload) = register_encoding_payload(&payload)?;
    let Some((name, path)) = readreg_parts(payload) else {
        return Err(invalid_readreg_payload());
    };
    let bytes = decode_buffer_bytes(fs::read(path)?, encoding);
    send_session_control_request(
        args,
        ScreenIpcRequest::SetRegister {
            name: name.to_string(),
            bytes,
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
    send_targeted_session_control_request(
        args,
        ScreenIpcRequest::PasteRegister {
            name: name.to_string(),
        },
    )
}

fn register_encoding_payload(payload: &str) -> io::Result<(Option<&'static Encoding>, &str)> {
    let payload = payload.trim_start();
    let Some(rest) = payload.strip_prefix("-e") else {
        return Ok((None, payload));
    };
    let (label, payload) = split_first_token(rest.trim_start())?;
    let encoding = Encoding::for_label(label.as_bytes()).ok_or_else(invalid_encoding_payload)?;
    Ok((Some(encoding), payload))
}

fn split_first_token(payload: &str) -> io::Result<(&str, &str)> {
    if payload.is_empty() {
        return Err(invalid_encoding_payload());
    }
    let token_end = payload.find(char::is_whitespace).unwrap_or(payload.len());
    let token = &payload[..token_end];
    let rest = payload[token_end..].trim_start();
    Ok((token, rest))
}

fn readreg_parts(payload: &str) -> Option<(&str, &str)> {
    let payload = payload.trim_start();
    let key_end = payload.find(char::is_whitespace)?;
    let key = &payload[..key_end];
    let path = payload[key_end..].trim();
    (!key.is_empty() && !path.is_empty()).then_some((key, path))
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

fn invalid_encoding_payload() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_buffer_encoding_required_hint(),
    )
}

fn invalid_readreg_payload() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_readreg_required_hint(),
    )
}
fn invalid_register_payload() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_register_required_hint(),
    )
}

#[cfg(test)]
mod tests {
    use super::{readreg_parts, register_encoding_payload, register_key, register_parts};

    #[test]
    fn extracts_register_parts_after_key() {
        assert_eq!(register_parts(". hello"), Some((".", "hello")));
        assert_eq!(register_parts("  a   echo hi"), Some(("a", "echo hi")));
        assert_eq!(register_parts("a"), None);
        assert_eq!(register_parts("  "), None);
    }

    #[test]
    fn extracts_readreg_parts_after_key() {
        assert_eq!(readreg_parts(". input.txt"), Some((".", "input.txt")));
        assert_eq!(readreg_parts("  a   path with spaces.txt"), Some(("a", "path with spaces.txt")));
        assert_eq!(readreg_parts("a"), None);
        assert_eq!(readreg_parts("  "), None);
    }
    #[test]
    fn extracts_process_register_key() {
        assert_eq!(register_key(". extra"), Some("."));
        assert_eq!(register_key("  a"), Some("a"));
        assert_eq!(register_key("  "), None);
    }

    #[test]
    fn parses_optional_register_encoding() {
        let (encoding, payload) = register_encoding_payload("-e windows-1252 . café").unwrap();

        assert_eq!(encoding.unwrap().name(), "windows-1252");
        assert_eq!(payload, ". café");
        assert!(register_encoding_payload("-e unknown . text").is_err());
    }
}