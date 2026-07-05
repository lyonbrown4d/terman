use std::{error::Error, io};

use crate::{
    args::{send_keys_args, target_session_name_arg},
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
    sessions::{BuiltinTmuxSession, load_builtin_tmux_sessions},
};

pub(crate) fn send_builtin_tmux_keys(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = target_session_name_arg(args).ok_or_else(target_required_error)?;
    let keys = send_keys_args(args);
    if keys.is_empty() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_tmux_keys_required_hint(),
        )));
    }
    let Some(session) = load_builtin_tmux_sessions()?
        .into_iter()
        .find(|session| session.name == target)
    else {
        return Err(session_not_found_error(&target));
    };

    match request_endpoint_response(
        &session_endpoint(&session),
        TmuxIpcRequest::Input {
            bytes: encode_send_keys(&keys),
        },
    )? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => {
            Err(Box::new(io::Error::new(io::ErrorKind::Unsupported, reason)))
        }
        response => Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")),
        ))),
    }
}

fn encode_send_keys(keys: &[String]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for key in keys {
        encode_key(key, &mut bytes);
    }
    bytes
}

fn encode_key(key: &str, bytes: &mut Vec<u8>) {
    match key {
        "Enter" | "C-m" => bytes.push(b'\r'),
        "C-j" => bytes.push(b'\n'),
        "Tab" | "C-i" => bytes.push(b'\t'),
        "Space" => bytes.push(b' '),
        "Escape" | "Esc" => bytes.push(0x1b),
        "Backspace" | "BSpace" => bytes.push(0x7f),
        "Delete" | "DC" => bytes.extend_from_slice(b"\x1b[3~"),
        "Up" => bytes.extend_from_slice(b"\x1b[A"),
        "Down" => bytes.extend_from_slice(b"\x1b[B"),
        "Right" => bytes.extend_from_slice(b"\x1b[C"),
        "Left" => bytes.extend_from_slice(b"\x1b[D"),
        key => encode_literal_or_control_key(key, bytes),
    }
}

fn encode_literal_or_control_key(key: &str, bytes: &mut Vec<u8>) {
    let mut chars = key.chars();
    if key.len() == 3 && key.starts_with("C-") {
        if let Some(ch) = chars.nth(2).and_then(control_byte) {
            bytes.push(ch);
            return;
        }
    }
    bytes.extend_from_slice(key.as_bytes());
}

fn control_byte(ch: char) -> Option<u8> {
    let lower = ch.to_ascii_lowercase();
    if lower.is_ascii_lowercase() {
        Some((lower as u8) - b'a' + 1)
    } else {
        None
    }
}

fn session_endpoint(session: &BuiltinTmuxSession) -> TmuxIpcEndpoint {
    session
        .ipc_endpoint
        .as_deref()
        .map(TmuxIpcEndpoint::from_raw_name)
        .unwrap_or_else(|| TmuxIpcEndpoint::for_session(&session.name))
}

fn target_required_error() -> Box<dyn Error> {
    Box::new(io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_tmux_target_required_hint(),
    ))
}

fn session_not_found_error(target: &str) -> Box<dyn Error> {
    Box::new(io::Error::new(
        io::ErrorKind::NotFound,
        terman_common::builtin_tmux_session_not_found_hint(target),
    ))
}

#[cfg(test)]
mod tests {
    use super::encode_send_keys;

    #[test]
    fn encodes_literals_and_common_keys() {
        assert_eq!(
            encode_send_keys(&["echo hi".into(), "Enter".into(), "C-c".into()]),
            b"echo hi\r\x03".to_vec()
        );
    }
}
