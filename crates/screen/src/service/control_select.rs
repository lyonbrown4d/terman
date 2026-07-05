use std::io;

use super::control_parse::control_command_payload;
use crate::{
    ScreenArgs,
    ipc::{ScreenIpcRequest, ScreenIpcResponse},
};

type SessionRequester = fn(&ScreenArgs, ScreenIpcRequest) -> io::Result<ScreenIpcResponse>;

pub(super) fn request_select_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
    request: SessionRequester,
) -> io::Result<()> {
    let selector = control_command_payload(inline_payload, extra_args);
    let selector = selector.trim();
    if !is_current_window_selector(selector) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_select_unsupported_hint(selector),
        ));
    }

    match request(args, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info { .. } => Ok(()),
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_screen_control_unexpected_response_hint(&format!(
                "{response:?}"
            )),
        )),
    }
}

fn is_current_window_selector(selector: &str) -> bool {
    matches!(selector, "" | "0" | "." | "#")
}

#[cfg(test)]
mod tests {
    use super::is_current_window_selector;

    #[test]
    fn accepts_single_window_selectors() {
        assert!(is_current_window_selector(""));
        assert!(is_current_window_selector("0"));
        assert!(is_current_window_selector("."));
        assert!(is_current_window_selector("#"));
        assert!(!is_current_window_selector("1"));
    }
}