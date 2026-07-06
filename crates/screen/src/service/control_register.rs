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

fn register_text(payload: &str) -> Option<&str> {
    let payload = payload.trim_start();
    let key_end = payload.find(char::is_whitespace)?;
    let key = &payload[..key_end];
    let text = payload[key_end..].trim_start();
    (!key.is_empty() && !text.is_empty()).then_some(text)
}

#[cfg(test)]
mod tests {
    use super::register_text;

    #[test]
    fn extracts_register_text_after_key() {
        assert_eq!(register_text(". hello"), Some("hello"));
        assert_eq!(register_text("  a   echo hi"), Some("echo hi"));
        assert_eq!(register_text("a"), None);
        assert_eq!(register_text("  "), None);
    }
}