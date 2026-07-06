use std::io;

use super::control_parse::control_command_payload;
use crate::{
    ScreenArgs,
    ipc::{ScreenIpcRequest, ScreenIpcResponse, ScreenWindowInfo},
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
    if selector == "-" {
        return handle_select_response(request(args, ScreenIpcRequest::LastWindow)?);
    }

    match request(args, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            active_window,
            windows,
            ..
        } => {
            let index = resolve_window_selector(selector, active_window, &windows)?;
            handle_select_response(request(args, ScreenIpcRequest::SelectWindow { index })?)
        }
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

fn resolve_window_selector(
    selector: &str,
    active_window: usize,
    windows: &[ScreenWindowInfo],
) -> io::Result<usize> {
    match selector {
        "" | "." | "#" => return Ok(active_window),
        _ => {}
    }

    if let Ok(index) = selector.parse::<usize>() {
        if windows.iter().any(|window| window.index == index) {
            return Ok(index);
        }
    }

    windows
        .iter()
        .find(|window| window.title == selector)
        .map(|window| window.index)
        .ok_or_else(|| unsupported_selector_error(selector))
}

fn handle_select_response(response: ScreenIpcResponse) -> io::Result<()> {
    match response {
        ScreenIpcResponse::Accepted => Ok(()),
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

fn unsupported_selector_error(selector: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_select_unsupported_hint(selector),
    )
}

fn unexpected_response_error(response: &ScreenIpcResponse) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        terman_common::builtin_screen_control_unexpected_response_hint(&format!("{response:?}")),
    )
}

#[cfg(test)]
mod tests {
    use super::resolve_window_selector;
    use crate::ipc::ScreenWindowInfo;

    fn windows() -> Vec<ScreenWindowInfo> {
        vec![
            ScreenWindowInfo {
                index: 0,
                title: String::from("shell"),
                active: false,
                replay_bytes: 0,
            },
            ScreenWindowInfo {
                index: 3,
                title: String::from("editor"),
                active: true,
                replay_bytes: 0,
            },
        ]
    }

    #[test]
    fn resolves_current_window_selectors() {
        let windows = windows();
        assert_eq!(resolve_window_selector("", 3, &windows).unwrap(), 3);
        assert_eq!(resolve_window_selector(".", 3, &windows).unwrap(), 3);
        assert_eq!(resolve_window_selector("#", 3, &windows).unwrap(), 3);
    }

    #[test]
    fn resolves_numeric_and_named_window_selectors() {
        let windows = windows();
        assert_eq!(resolve_window_selector("0", 3, &windows).unwrap(), 0);
        assert_eq!(resolve_window_selector("editor", 0, &windows).unwrap(), 3);
        assert!(resolve_window_selector("missing", 0, &windows).is_err());
    }
}