use std::{fs, io::{self, Write}};

use super::ipc_client::request_endpoint_response;
use crate::ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse};

const DEFAULT_ATTACH_HARDCOPY_PATH: &str = "hardcopy.0";

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
            fs::write(DEFAULT_ATTACH_HARDCOPY_PATH, &bytes)?;
            let mut stdout = io::stdout();
            stdout.write_all(b"\r\n")?;
            stdout.write_all(
                terman_common::builtin_screen_control_hardcopy_complete_hint(
                    DEFAULT_ATTACH_HARDCOPY_PATH,
                    bytes.len(),
                )
                .as_bytes(),
            )?;
            stdout.write_all(b"\r\n")?;
            stdout.flush()
        }
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected screen attach hardcopy response",
        )),
    }
}

pub(super) fn print_attach_info(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    match request_endpoint_response(endpoint, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            replay_bytes,
            attach_clients,
            cols,
            rows,
        } => {
            let mut stdout = io::stdout();
            stdout.write_all(b"\r\n")?;
            stdout.write_all(
                terman_common::builtin_screen_control_info_hint(
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
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected screen attach info response",
        )),
    }
}