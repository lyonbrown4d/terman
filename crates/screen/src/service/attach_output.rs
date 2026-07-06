use std::{
    env,
    fs::OpenOptions,
    io::{self, Write},
};

use super::{control_time::screen_time_message, ipc_client::request_endpoint_response};
use crate::ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse};

const ATTACH_HARDCOPY_PREFIX_ENV: &str = "TERMAN_SCREEN_HARDCOPY_PREFIX";
const DEFAULT_ATTACH_HARDCOPY_PREFIX: &str = "hardcopy";
const MAX_ATTACH_HARDCOPY_SLOTS: usize = 10_000;

pub(super) fn print_attach_help() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(b"\r\n")?;
    stdout.write_all(terman_common::builtin_screen_attach_help_hint().as_bytes())?;
    stdout.write_all(b"\r\n")?;
    stdout.flush()
}

pub(super) fn print_attach_hardcopy(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    match request_endpoint_response(endpoint, ScreenIpcRequest::Hardcopy)? {
        ScreenIpcResponse::Hardcopy { bytes } => {
            let path = write_numbered_hardcopy(&bytes)?;
            let mut stdout = io::stdout();
            stdout.write_all(b"\r\n")?;
            stdout.write_all(
                terman_common::builtin_screen_control_hardcopy_complete_hint(&path, bytes.len())
                    .as_bytes(),
            )?;
            stdout.write_all(b"\r\n")?;
            stdout.flush()
        }
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

pub(super) fn print_attach_info(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    match request_endpoint_response(endpoint, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            session_name,
            replay_bytes,
            attach_clients,
            cols,
            rows,
        } => {
            let mut stdout = io::stdout();
            stdout.write_all(b"\r\n")?;
            stdout.write_all(
                terman_common::builtin_screen_control_info_hint(
                    &session_name,
                    replay_bytes,
                    attach_clients,
                    cols,
                    rows,
                )
                .as_bytes(),
            )?;
            stdout.write_all(b"\r\n")?;
            stdout.flush()
        }
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

pub(super) fn print_attach_time() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(b"\r\n")?;
    stdout.write_all(screen_time_message().as_bytes())?;
    stdout.write_all(b"\r\n")?;
    stdout.flush()
}

pub(super) fn print_attach_version() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(b"\r\n")?;
    stdout.write_all(
        terman_common::builtin_screen_control_version_hint(env!("CARGO_PKG_VERSION")).as_bytes(),
    )?;
    stdout.write_all(b"\r\n")?;
    stdout.flush()
}

pub(super) fn print_attach_displays(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    match request_endpoint_response(endpoint, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            session_name,
            attach_clients,
            cols,
            rows,
            ..
        } => {
            let mut stdout = io::stdout();
            stdout.write_all(b"\r\n")?;
            stdout.write_all(
                terman_common::builtin_screen_control_displays_entry_hint(
                    &session_name,
                    attach_clients,
                    cols,
                    rows,
                )
                .as_bytes(),
            )?;
            stdout.write_all(b"\r\n")?;
            stdout.flush()
        }
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

pub(super) fn print_attach_windows(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    match request_endpoint_response(endpoint, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            session_name,
            replay_bytes,
            attach_clients,
            cols,
            rows,
        } => {
            let mut stdout = io::stdout();
            stdout.write_all(b"\r\n")?;
            stdout.write_all(
                terman_common::builtin_screen_control_windows_entry_hint(
                    &session_name,
                    replay_bytes,
                    attach_clients,
                    cols,
                    rows,
                )
                .as_bytes(),
            )?;
            stdout.write_all(b"\r\n")?;
            stdout.flush()
        }
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

fn write_numbered_hardcopy(bytes: &[u8]) -> io::Result<String> {
    let prefix = attach_hardcopy_prefix();
    for index in 0..MAX_ATTACH_HARDCOPY_SLOTS {
        let path = attach_hardcopy_path(&prefix, index);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                file.write_all(bytes)?;
                return Ok(path);
            }
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {}
            Err(err) => return Err(err),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        terman_common::builtin_screen_attach_hardcopy_path_unavailable_hint(),
    ))
}

fn unexpected_response_error(response: &ScreenIpcResponse) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        terman_common::builtin_screen_unexpected_response_hint(&format!("{response:?}")),
    )
}

fn attach_hardcopy_prefix() -> String {
    env::var(ATTACH_HARDCOPY_PREFIX_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_ATTACH_HARDCOPY_PREFIX.to_string())
}

fn attach_hardcopy_path(prefix: &str, index: usize) -> String {
    format!("{prefix}.{index}")
}

#[cfg(test)]
mod tests {
    use super::attach_hardcopy_path;

    #[test]
    fn formats_attach_hardcopy_path() {
        assert_eq!(attach_hardcopy_path("hardcopy", 0), "hardcopy.0");
        assert_eq!(attach_hardcopy_path("screen-copy", 42), "screen-copy.42");
    }
}
