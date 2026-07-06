use std::io;

use super::{
    control_parse::{control_command_payload, decode_stuff_payload},
    control_session::{request_paste_command, send_session_control_request},
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
    let Some(text) = register_text(&payload) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_stuff_required_hint(),
        ));
    };
    send_session_control_request(
        args,
        ScreenIpcRequest::SetPasteBuffer {
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
    let Some(_) = register_key(&payload) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_stuff_required_hint(),
        ));
    };
    request_paste_command(args, "", &[])
}
fn register_text(payload: &str) -> Option<&str> {
    let payload = payload.trim_start();
    let key_end = payload.find(char::is_whitespace)?;
    let key = &payload[..key_end];
    let text = payload[key_end..].trim_start();
    (!key.is_empty() && !text.is_empty()).then_some(text)
}
fn register_key(payload: &str) -> Option<&str> {
    payload.split_whitespace().next()
}
#[cfg(test)]
mod tests {
    use super::{register_key, register_text};

    #[test]
    fn extracts_register_text_after_key() {
        assert_eq!(register_text(". hello"), Some("hello"));
        assert_eq!(register_text("  a   echo hi"), Some("echo hi"));
        assert_eq!(register_text("a"), None);
        assert_eq!(register_text("  "), None);
    }
    #[test]
    fn extracts_process_register_key() {
        assert_eq!(register_key(". extra"), Some("."));
        assert_eq!(register_key("  a"), Some("a"));
        assert_eq!(register_key("  "), None);
    }
}