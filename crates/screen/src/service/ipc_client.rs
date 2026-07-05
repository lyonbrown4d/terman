use std::io::{self, BufRead, BufReader, Write};

use crate::ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse};

pub(super) fn send_control_request(
    endpoint: &ScreenIpcEndpoint,
    request: ScreenIpcRequest,
) -> io::Result<()> {
    match request_endpoint_response(endpoint, request)? {
        ScreenIpcResponse::Accepted => Ok(()),
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

pub(super) fn request_endpoint_response(
    endpoint: &ScreenIpcEndpoint,
    request: ScreenIpcRequest,
) -> io::Result<ScreenIpcResponse> {
    let mut stream = endpoint.connect_options()?.connect_sync()?;
    serde_json::to_writer(&mut stream, &request)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut response = String::new();
    BufReader::new(stream).read_line(&mut response)?;
    serde_json::from_str(response.trim_end())
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}
