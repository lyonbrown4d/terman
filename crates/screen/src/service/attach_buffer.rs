use std::{fs, io, path::PathBuf};

use super::ipc_client::{request_endpoint_response, send_control_request};
use crate::ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse};

pub(super) fn read_attach_buffer(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    let path = current_buffer_file(endpoint)?;
    let bytes = fs::read(path)?;
    send_control_request(endpoint, ScreenIpcRequest::SetPasteBuffer { bytes })
}

pub(super) fn remove_attach_buffer(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    let path = current_buffer_file(endpoint)?;
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }
    send_control_request(
        endpoint,
        ScreenIpcRequest::SetPasteBuffer { bytes: Vec::new() },
    )
}

pub(super) fn write_attach_buffer(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    let path = current_buffer_file(endpoint)?;
    match request_endpoint_response(endpoint, ScreenIpcRequest::GetPasteBuffer)? {
        ScreenIpcResponse::PasteBuffer { bytes } => {
            fs::write(&path, &bytes)?;
            print_writebuf_complete(&path, bytes.len());
            Ok(())
        }
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

fn current_buffer_file(endpoint: &ScreenIpcEndpoint) -> io::Result<PathBuf> {
    match request_endpoint_response(endpoint, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info { buffer_file, .. } => Ok(buffer_file),
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

fn print_writebuf_complete(path: &PathBuf, bytes: usize) {
    let path = path.display().to_string();
    println!(
        "{}",
        terman_common::builtin_screen_control_writebuf_complete_hint(&path, bytes)
    );
}

fn unexpected_response_error(response: &ScreenIpcResponse) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        terman_common::builtin_screen_control_unexpected_response_hint(&format!("{response:?}")),
    )
}