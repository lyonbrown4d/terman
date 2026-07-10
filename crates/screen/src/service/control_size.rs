use std::io;

use super::{
    control_parse::control_command_payload,
    control_session::{request_session_response, send_session_control_request},
};
use crate::{
    ScreenArgs,
    ipc::{ScreenIpcRequest, ScreenIpcResponse},
};

struct SizeSpec {
    primary: Option<u16>,
    secondary: Option<u16>,
}

pub(super) fn request_fit_command(args: &ScreenArgs) -> io::Result<()> {
    let (cols, rows) = terman_common::current_terminal_size()?;
    send_session_control_request(args, ScreenIpcRequest::Resize { cols, rows })
}

pub(super) fn request_size_command(
    args: &ScreenArgs,
    command: &str,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let spec = parse_size_payload(&payload)?;
    let (current_cols, current_rows) = session_size(args)?;
    let request = match command {
        "width" => ScreenIpcRequest::Resize {
            cols: spec.primary.unwrap_or_else(|| toggled_width(current_cols)),
            rows: spec.secondary.unwrap_or(current_rows),
        },
        "height" => ScreenIpcRequest::Resize {
            cols: spec.secondary.unwrap_or(current_cols),
            rows: spec.primary.unwrap_or_else(|| toggled_height(current_rows)),
        },
        _ => return Ok(()),
    };
    send_session_control_request(args, request)
}

fn parse_size_payload(payload: &str) -> io::Result<SizeSpec> {
    let mut parts = payload.split_whitespace();
    let first = parts.next();
    let first = match first {
        Some("-w") | Some("-d") => parts.next(),
        value => value,
    };
    let primary = first.map(parse_dimension).transpose()?;
    let secondary = parts.next().map(parse_dimension).transpose()?;
    if parts.next().is_some() {
        return Err(invalid_size_payload());
    }
    Ok(SizeSpec { primary, secondary })
}

fn parse_dimension(value: &str) -> io::Result<u16> {
    match value.parse::<u16>() {
        Ok(value) if value > 0 => Ok(value),
        _ => Err(invalid_size_payload()),
    }
}

fn session_size(args: &ScreenArgs) -> io::Result<(u16, u16)> {
    match request_session_response(args, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info { cols, rows, .. } => {
            Ok((cols.unwrap_or(80), rows.unwrap_or(24)))
        }
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

fn toggled_width(cols: u16) -> u16 {
    if cols == 80 { 132 } else { 80 }
}

fn toggled_height(rows: u16) -> u16 {
    if rows == 24 { 42 } else { 24 }
}

fn invalid_size_payload() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_size_required_hint(),
    )
}

#[cfg(test)]
mod tests {
    use super::{parse_size_payload, toggled_height, toggled_width};

    #[test]
    fn parses_width_and_height_payloads() {
        let spec = parse_size_payload("-w 132 40").unwrap();

        assert_eq!(spec.primary, Some(132));
        assert_eq!(spec.secondary, Some(40));
        assert!(parse_size_payload("0").is_err());
        assert!(parse_size_payload("80 24 extra").is_err());
    }

    #[test]
    fn toggles_screen_defaults() {
        assert_eq!(toggled_width(80), 132);
        assert_eq!(toggled_width(132), 80);
        assert_eq!(toggled_height(24), 42);
        assert_eq!(toggled_height(42), 24);
    }
}