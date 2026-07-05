use std::{
    fs::OpenOptions,
    io::{self, Write},
};

use super::ipc_client::request_endpoint_response;
use crate::ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse};

const ATTACH_HARDCOPY_BASENAME: &str = "hardcopy";
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

fn write_numbered_hardcopy(bytes: &[u8]) -> io::Result<String> {
    for index in 0..MAX_ATTACH_HARDCOPY_SLOTS {
        let path = attach_hardcopy_path(index);
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
        "no available screen attach hardcopy path",
    ))
}

fn attach_hardcopy_path(index: usize) -> String {
    format!("{ATTACH_HARDCOPY_BASENAME}.{index}")
}

#[cfg(test)]
mod tests {
    use super::attach_hardcopy_path;

    #[test]
    fn formats_attach_hardcopy_path() {
        assert_eq!(attach_hardcopy_path(0), "hardcopy.0");
        assert_eq!(attach_hardcopy_path(42), "hardcopy.42");
    }
}