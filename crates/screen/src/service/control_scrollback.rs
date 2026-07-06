use std::io;

use super::{
    control_parse::control_command_payload,
    control_session::{send_session_control_request, send_targeted_session_control_request},
};
use crate::{ScreenArgs, ipc::ScreenIpcRequest};

pub(super) fn request_scrollback_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let lines = parse_scrollback_lines(inline_payload, extra_args)?;
    send_targeted_session_control_request(args, ScreenIpcRequest::SetScrollback { lines })
}

pub(super) fn request_defscrollback_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let lines = parse_scrollback_lines(inline_payload, extra_args)?;
    send_session_control_request(args, ScreenIpcRequest::SetDefaultScrollback { lines })
}

fn parse_scrollback_lines(inline_payload: &str, extra_args: &[String]) -> io::Result<usize> {
    let payload = control_command_payload(inline_payload, extra_args);
    payload.trim().parse::<usize>().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_scrollback_required_hint(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::parse_scrollback_lines;

    #[test]
    fn parses_scrollback_lines() {
        assert_eq!(parse_scrollback_lines("1200", &[]).unwrap(), 1200);
        assert_eq!(parse_scrollback_lines("", &["2400".into()]).unwrap(), 2400);
        assert!(parse_scrollback_lines("abc", &[]).is_err());
    }
}