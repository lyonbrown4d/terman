use std::io::{self, BufRead, BufReader, Write};

use crate::ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse};

use super::store::BuiltinScreenSession;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BuiltinScreenSessionRuntimeStatus {
    pub(crate) replay_bytes: usize,
    pub(crate) attach_clients: usize,
    pub(crate) cols: Option<u16>,
    pub(crate) rows: Option<u16>,
}

pub(crate) fn load_builtin_screen_runtime_status(
    session: &BuiltinScreenSession,
) -> io::Result<BuiltinScreenSessionRuntimeStatus> {
    let endpoint = builtin_screen_session_endpoint(session);
    let mut stream = endpoint.connect_options()?.connect_sync()?;
    serde_json::to_writer(&mut stream, &ScreenIpcRequest::Info)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut response = String::new();
    BufReader::new(stream).read_line(&mut response)?;
    match serde_json::from_str(response.trim_end())
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?
    {
        ScreenIpcResponse::Info {
            replay_bytes,
            attach_clients,
            cols,
            rows,
            ..
        } => Ok(BuiltinScreenSessionRuntimeStatus {
            replay_bytes,
            attach_clients,
            cols,
            rows,
        }),
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_screen_unexpected_response_hint(&format!("{response:?}")),
        )),
    }
}

fn builtin_screen_session_endpoint(session: &BuiltinScreenSession) -> ScreenIpcEndpoint {
    session
        .ipc_endpoint
        .as_deref()
        .map(ScreenIpcEndpoint::from_raw_name)
        .unwrap_or_else(|| ScreenIpcEndpoint::for_session(&session.name))
}
