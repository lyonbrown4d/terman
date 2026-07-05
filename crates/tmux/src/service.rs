use std::io::{self, BufRead, BufReader, Write};

use interprocess::local_socket::prelude::*;

use crate::ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse};

#[allow(dead_code)]
pub(crate) fn request_endpoint_response(
    endpoint: &TmuxIpcEndpoint,
    request: TmuxIpcRequest,
) -> io::Result<TmuxIpcResponse> {
    let mut stream = endpoint.connect_options()?.connect_sync()?;
    write_request(&mut stream, &request)?;
    read_response(stream)
}

fn write_request(stream: &mut LocalSocketStream, request: &TmuxIpcRequest) -> io::Result<()> {
    serde_json::to_writer(&mut *stream, request)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    stream.write_all(b"\n")?;
    stream.flush()
}

fn read_response(stream: LocalSocketStream) -> io::Result<TmuxIpcResponse> {
    let mut response = String::new();
    let mut reader = BufReader::new(stream);
    reader.read_line(&mut response)?;
    serde_json::from_str(response.trim_end())
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

#[cfg(test)]
mod tests {
    use crate::ipc::TmuxIpcRequest;

    #[test]
    fn models_client_request_payload() {
        assert_eq!(TmuxIpcRequest::Ping, TmuxIpcRequest::Ping);
    }
}

