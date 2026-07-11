use std::io;

use super::{
    control_parse::control_command_payload,
    control_session::send_session_control_request,
};
use crate::{
    ScreenArgs,
    ipc::ScreenIpcRequest,
    region_types::{ScreenRegionResize, ScreenRegionResizeAmount, ScreenRegionResizeMode},
};

pub(super) fn request_region_resize_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let resize = parse_region_resize(payload.as_str())?;
    send_session_control_request(args, ScreenIpcRequest::ResizeRegion { resize })
}

fn parse_region_resize(payload: &str) -> io::Result<ScreenRegionResize> {
    let mut parts = payload.split_whitespace();
    let first = parts.next().ok_or_else(invalid_resize_payload)?;
    let (mode, amount) = match first {
        "-h" => (ScreenRegionResizeMode::Width, parts.next()),
        "-v" => (ScreenRegionResizeMode::Height, parts.next()),
        "-b" => (ScreenRegionResizeMode::Both, parts.next()),
        "-l" => (ScreenRegionResizeMode::Local, parts.next()),
        "-p" => (ScreenRegionResizeMode::Perpendicular, parts.next()),
        amount => (ScreenRegionResizeMode::Local, Some(amount)),
    };
    let amount = amount.ok_or_else(invalid_resize_payload)?;
    if parts.next().is_some() { return Err(invalid_resize_payload()); }
    Ok(ScreenRegionResize { mode, amount: parse_amount(amount)? })
}

fn parse_amount(value: &str) -> io::Result<ScreenRegionResizeAmount> {
    match value {
        "=" => return Ok(ScreenRegionResizeAmount::Equalize),
        "max" | "_" => return Ok(ScreenRegionResizeAmount::Maximum),
        "min" | "0" => return Ok(ScreenRegionResizeAmount::Minimum),
        _ => {}
    }
    if let Some(percent) = value.strip_suffix('%') {
        return parse_percent(percent);
    }
    if let Some(delta) = value.strip_prefix('+') {
        return parse_i32(delta).map(ScreenRegionResizeAmount::Delta);
    }
    if let Some(delta) = value.strip_prefix('-') {
        return parse_i32(delta).map(|delta| ScreenRegionResizeAmount::Delta(-delta));
    }
    value.parse::<u16>()
        .ok().filter(|value| *value > 0)
        .map(ScreenRegionResizeAmount::Absolute)
        .ok_or_else(invalid_resize_payload)
}

fn parse_percent(value: &str) -> io::Result<ScreenRegionResizeAmount> {
    if let Some(delta) = value.strip_prefix('+') {
        return parse_i32(delta).map(ScreenRegionResizeAmount::DeltaPercent);
    }
    if let Some(delta) = value.strip_prefix('-') {
        return parse_i32(delta).map(|delta| ScreenRegionResizeAmount::DeltaPercent(-delta));
    }
    value.parse::<u16>()
        .ok().filter(|value| *value <= 100)
        .map(ScreenRegionResizeAmount::AbsolutePercent)
        .ok_or_else(invalid_resize_payload)
}

fn parse_i32(value: &str) -> io::Result<i32> {
    value.parse::<i32>()
        .ok().filter(|value| *value >= 0)
        .ok_or_else(invalid_resize_payload)
}

fn invalid_resize_payload() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_resize_required_hint(),
    )
}