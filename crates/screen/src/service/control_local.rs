use std::io;

use super::{
    control_help::request_help_command,
    control_sleep::request_sleep_command,
    control_time::request_time_command,
    control_version::request_version_command,
};

pub(super) fn request_local_control_command(
    command: &str,
    inline_payload: &str,
    extra_args: &[String],
) -> Option<io::Result<()>> {
    match command {
        "help" | "commands" => Some(request_help()),
        "sleep" => Some(request_sleep_command(inline_payload, extra_args)),
        "time" => Some(request_time()),
        "version" => Some(request_version()),
        _ => None,
    }
}

fn request_help() -> io::Result<()> {
    request_help_command();
    Ok(())
}

fn request_time() -> io::Result<()> {
    request_time_command();
    Ok(())
}

fn request_version() -> io::Result<()> {
    request_version_command();
    Ok(())
}