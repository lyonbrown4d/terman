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
    match request(args, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            active_window,
            windows,
            ..
        } => {
            let index = parse_window_selector(selector, active_window)?;
            if !windows.iter().any(|window| window.index == index) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    terman_common::builtin_screen_control_select_unsupported_hint(selector),
                ));
            }
            match request(args, ScreenIpcRequest::SelectWindow { index })? {
                ScreenIpcResponse::Accepted => Ok(()),
                ScreenIpcResponse::Rejected { reason } => {
                    Err(io::Error::new(io::ErrorKind::Unsupported, reason))
                }
                response => Err(unexpected_response_error(&response)),
            }
        }
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

fn parse_window_selector(selector: &str, active_window: usize) -> io::Result<usize> {
    match selector {
        "" | "." | "#" => Ok(active_window),
        value => value.parse::<usize>().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                terman_common::builtin_screen_control_select_unsupported_hint(selector),
            )
        }),
    }
}

fn unexpected_response_error(response: &ScreenIpcResponse) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        terman_common::builtin_screen_control_unexpected_response_hint(&format!("{response:?}")),
    )
}

#[cfg(test)]
mod tests {
    use super::parse_window_selector;

    #[test]
    fn parses_current_window_selectors() {
        assert_eq!(parse_window_selector("", 2).unwrap(), 2);
        assert_eq!(parse_window_selector(".", 2).unwrap(), 2);
        assert_eq!(parse_window_selector("#", 2).unwrap(), 2);
    }

    #[test]
    fn parses_numeric_window_selector() {
        assert_eq!(parse_window_selector("3", 0).unwrap(), 3);
        assert!(parse_window_selector("name", 0).is_err());
    }
}