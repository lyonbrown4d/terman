use std::{
    error::Error,
    io::{self, BufRead, BufReader, Write},
};

use interprocess::local_socket::prelude::*;

use crate::{
    args::target_session_arg,
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    sessions::load_builtin_tmux_sessions,
};

pub(crate) fn attach_builtin_tmux_session(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_arg(args)?;
    let endpoint = find_session_endpoint(&target)?;

    stream_attach_output(&endpoint)
}

fn required_target_session_arg(args: &[String]) -> Result<String, Box<dyn Error>> {
    target_session_arg(args).ok_or_else(|| {
        Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_tmux_target_required_hint(),
        )) as Box<dyn Error>
    })
}

fn find_session_endpoint(target: &str) -> Result<TmuxIpcEndpoint, Box<dyn Error>> {
    let Some(session) = load_builtin_tmux_sessions()?
        .into_iter()
        .find(|session| session.name == target)
    else {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            terman_common::builtin_tmux_session_not_found_hint(target),
        )));
    };

    Ok(session
        .ipc_endpoint
        .as_deref()
        .map(TmuxIpcEndpoint::from_raw_name)
        .unwrap_or_else(|| TmuxIpcEndpoint::for_session(&session.name)))
}

fn stream_attach_output(endpoint: &TmuxIpcEndpoint) -> Result<(), Box<dyn Error>> {
    let stream = open_attach_stream(endpoint)?;
    let mut reader = BufReader::new(stream);
    let mut stdout = io::stdout().lock();

    loop {
        let Some(response) = read_attach_response(&mut reader)? else {
            return Ok(());
        };

        match response {
            TmuxIpcResponse::Attached { replay } => write_output(&mut stdout, &replay)?,
            TmuxIpcResponse::Output { bytes } => write_output(&mut stdout, &bytes)?,
            TmuxIpcResponse::Detached => return Ok(()),
            TmuxIpcResponse::Exit { .. } => return Ok(()),
            TmuxIpcResponse::Rejected { reason } => {
                return Err(Box::new(io::Error::new(io::ErrorKind::PermissionDenied, reason)));
            }
            TmuxIpcResponse::Accepted
            | TmuxIpcResponse::Info { .. }
            | TmuxIpcResponse::Resize { .. } => {}
        }
    }
}

fn open_attach_stream(endpoint: &TmuxIpcEndpoint) -> io::Result<LocalSocketStream> {
    let mut stream = endpoint.connect_options()?.connect_sync()?;
    write_attach_request(&mut stream)?;
    Ok(stream)
}

fn write_attach_request(stream: &mut LocalSocketStream) -> io::Result<()> {
    serde_json::to_writer(&mut *stream, &TmuxIpcRequest::Attach).map_err(json_io_error)?;
    stream.write_all(b"\n")?;
    stream.flush()
}

fn read_attach_response(
    reader: &mut BufReader<LocalSocketStream>,
) -> io::Result<Option<TmuxIpcResponse>> {
    let mut line = String::new();
    let bytes = reader.read_line(&mut line)?;
    if bytes == 0 {
        return Ok(None);
    }

    serde_json::from_str(line.trim_end())
        .map(Some)
        .map_err(json_io_error)
}

fn write_output(stdout: &mut dyn Write, bytes: &[u8]) -> io::Result<()> {
    stdout.write_all(bytes)?;
    stdout.flush()
}

fn json_io_error(error: serde_json::Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}