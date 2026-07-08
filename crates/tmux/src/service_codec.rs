use std::io::{self, BufRead, BufReader, Write};

use interprocess::local_socket::prelude::*;

use crate::ipc::{TmuxIpcRequest, TmuxIpcResponse};

pub(crate) fn write_request(stream: &mut LocalSocketStream, request: &TmuxIpcRequest) -> io::Result<()> {
    serde_json::to_writer(&mut *stream, request)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    stream.write_all(b"\n")?;
    stream.flush()
}

pub(crate) fn read_response(stream: LocalSocketStream) -> io::Result<TmuxIpcResponse> {
    let mut response = String::new();
    let mut reader = BufReader::new(stream);
    reader.read_line(&mut response)?;
    serde_json::from_str(response.trim_end())
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

pub(crate) fn write_response(stream: &mut LocalSocketStream, response: &TmuxIpcResponse) -> io::Result<()> {
    serde_json::to_writer(&mut *stream, response)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    stream.write_all(b"\n")?;
    stream.flush()
}
