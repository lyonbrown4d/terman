use std::io::{self, Write};

use super::{
    attach_hardcopy::{write_numbered_hardcopy, AttachHardcopySettings},
    control_time::screen_time_message,
    ipc_client::request_endpoint_response,
};
use crate::ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse, ScreenWindowInfo};

pub(super) fn print_attach_help() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(b"\r\n")?;
    stdout.write_all(terman_common::builtin_screen_attach_help_hint().as_bytes())?;
    stdout.write_all(b"\r\n")?;
    stdout.flush()
}

pub(super) fn print_attach_hardcopy(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    let settings = attach_hardcopy_settings(endpoint)?;
    match request_endpoint_response(
        endpoint,
        ScreenIpcRequest::Hardcopy {
            include_history: false,
        },
    )? {
        ScreenIpcResponse::Hardcopy { bytes } => {
            let path = write_numbered_hardcopy(&settings, &bytes)?;
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
            scrollback_lines,
            ..
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
                    scrollback_lines,
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

pub(super) fn print_attach_license() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(b"\r\n")?;
    stdout.write_all(
        terman_common::builtin_screen_control_license_hint(env!("CARGO_PKG_VERSION")).as_bytes(),
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
            attach_clients,
            cols,
            rows,
            windows,
            ..
        } => {
            let mut stdout = io::stdout();
            stdout.write_all(b"\r\n")?;
            for window in windows {
                write_attach_window_entry(&mut stdout, &window, attach_clients, cols, rows)?;
            }
            stdout.flush()
        }
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

fn attach_hardcopy_settings(endpoint: &ScreenIpcEndpoint) -> io::Result<AttachHardcopySettings> {
    match request_endpoint_response(endpoint, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            active_window,
            hardcopy_append,
            hardcopy_dir,
            ..
        } => Ok(AttachHardcopySettings {
            append: hardcopy_append,
            directory: hardcopy_dir,
            window_index: active_window,
        }),
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

fn write_attach_window_entry(
    stdout: &mut impl Write,
    window: &ScreenWindowInfo,
    attach_clients: usize,
    cols: Option<u16>,
    rows: Option<u16>,
) -> io::Result<()> {
    stdout.write_all(
        terman_common::builtin_screen_control_windows_entry_hint(
            window.index,
            window.active,
            &window.title,
            window.replay_bytes,
            attach_clients,
            cols,
            rows,
        )
        .as_bytes(),
    )?;
    stdout.write_all(b"\r\n")
}

fn unexpected_response_error(response: &ScreenIpcResponse) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        terman_common::builtin_screen_unexpected_response_hint(&format!("{response:?}")),
    )
}
