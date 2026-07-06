use std::{fs, io, path::PathBuf};

use super::{
    control_buffer_encoding::{decode_buffer_bytes, encode_buffer_bytes, parse_buffer_io_spec},
    control_parse::control_command_payload,
    control_session::{request_session_response, send_session_control_request},
};
use crate::{
    ScreenArgs,
    ipc::{ScreenIpcRequest, ScreenIpcResponse},
    screen_exchange::default_screen_exchange_file,
};

pub(super) fn request_bufferfile_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let path = buffer_path_or_default(&payload);
    send_session_control_request(args, ScreenIpcRequest::SetBufferFile { path })
}

pub(super) fn request_readbuf_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let spec = parse_buffer_io_spec(&payload)?;
    let path = buffer_path(args, spec.path.as_ref())?;
    let bytes = decode_buffer_bytes(fs::read(path)?, spec.encoding);
    send_session_control_request(args, ScreenIpcRequest::SetPasteBuffer { bytes })
}

pub(super) fn request_removebuf_command(args: &ScreenArgs) -> io::Result<()> {
    let path = current_buffer_file(args)?;
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }
    send_session_control_request(args, ScreenIpcRequest::SetPasteBuffer { bytes: Vec::new() })
}

pub(super) fn request_writebuf_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let spec = parse_buffer_io_spec(&payload)?;
    let path = buffer_path(args, spec.path.as_ref())?;
    request_session_writebuf(args, &path, spec.encoding)
}

fn buffer_path(args: &ScreenArgs, path: Option<&PathBuf>) -> io::Result<PathBuf> {
    match path {
        Some(path) => Ok(path.clone()),
        None => current_buffer_file(args),
    }
}

fn buffer_path_or_default(payload: &str) -> PathBuf {
    let payload = payload.trim();
    if payload.is_empty() {
        default_screen_exchange_file()
    } else {
        PathBuf::from(payload)
    }
}

fn current_buffer_file(args: &ScreenArgs) -> io::Result<PathBuf> {
    match request_session_response(args, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info { buffer_file, .. } => Ok(buffer_file),
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

fn request_session_writebuf(
    args: &ScreenArgs,
    path: &PathBuf,
    encoding: Option<&'static encoding_rs::Encoding>,
) -> io::Result<()> {
    match request_session_response(args, ScreenIpcRequest::GetPasteBuffer)? {
        ScreenIpcResponse::PasteBuffer { bytes } => {
            let output = encode_buffer_bytes(&bytes, encoding);
            fs::write(path, &output)?;
            let bytes = output.len();
            let path = path.display().to_string();
            println!(
                "{}",
                terman_common::builtin_screen_control_writebuf_complete_hint(&path, bytes)
            );
            Ok(())
        }
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

fn unexpected_response_error(response: &ScreenIpcResponse) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        terman_common::builtin_screen_control_unexpected_response_hint(&format!("{response:?}")),
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::buffer_path_or_default;
    use crate::screen_exchange::default_screen_exchange_file;

    #[test]
    fn parses_bufferfile_path_or_default() {
        assert_eq!(buffer_path_or_default(""), default_screen_exchange_file());
        assert_eq!(buffer_path_or_default("exchange.txt"), PathBuf::from("exchange.txt"));
    }
}