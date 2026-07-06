use std::io;

use super::{
    control_parse::control_command_payload,
    control_session::{request_session_response, send_session_control_request},
    control_target::resolve_window_selector,
};
use crate::{
    ScreenArgs,
    ipc::{ScreenIpcRequest, ScreenIpcResponse, ScreenWindowInfo},
};

pub(super) fn request_number_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let payload = payload.trim();
    let (active_window, windows) = session_window_info(args)?;
    let source = selected_window(args, active_window, &windows)?;
    if payload.is_empty() {
        print_window_number(source, &windows);
        return Ok(());
    }
    let index = parse_number_target(payload, source)?;
    send_session_control_request(args, ScreenIpcRequest::NumberWindow { source, index })
}

fn session_window_info(args: &ScreenArgs) -> io::Result<(usize, Vec<ScreenWindowInfo>)> {
    match request_session_response(args, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            active_window,
            windows,
            ..
        } => Ok((active_window, windows)),
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

fn selected_window(
    args: &ScreenArgs,
    active_window: usize,
    windows: &[ScreenWindowInfo],
) -> io::Result<usize> {
    match args.window_selector.as_deref().map(str::trim) {
        Some(selector) if !selector.is_empty() => {
            resolve_window_selector(selector, active_window, windows)
        }
        _ => Ok(active_window),
    }
}

fn print_window_number(index: usize, windows: &[ScreenWindowInfo]) {
    let title = windows
        .iter()
        .find(|window| window.index == index)
        .map(|window| window.title.as_str())
        .unwrap_or_default();
    println!("{}", terman_common::builtin_screen_control_number_hint(index, title));
}

fn parse_number_target(payload: &str, source: usize) -> io::Result<usize> {
    if let Some(delta) = payload.strip_prefix('+') {
        return source
            .checked_add(parse_number_delta(delta)?)
            .ok_or_else(invalid_number_payload);
    }
    if let Some(delta) = payload.strip_prefix('-') {
        return source
            .checked_sub(parse_number_delta(delta)?)
            .ok_or_else(invalid_number_payload);
    }
    payload.parse::<usize>().map_err(|_| invalid_number_payload())
}

fn parse_number_delta(delta: &str) -> io::Result<usize> {
    if delta.trim().is_empty() {
        return Err(invalid_number_payload());
    }
    delta.parse::<usize>().map_err(|_| invalid_number_payload())
}

fn invalid_number_payload() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_number_invalid_hint(),
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
    use super::parse_number_target;

    #[test]
    fn parses_absolute_and_relative_number_targets() {
        assert_eq!(parse_number_target("2", 3).unwrap(), 2);
        assert_eq!(parse_number_target("+2", 3).unwrap(), 5);
        assert_eq!(parse_number_target("-2", 3).unwrap(), 1);
        assert!(parse_number_target("-4", 3).is_err());
        assert!(parse_number_target("x", 3).is_err());
    }
}