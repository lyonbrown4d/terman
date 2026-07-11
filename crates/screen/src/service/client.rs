use std::{
    io::{self, Write},
    process,
    time::{SystemTime, UNIX_EPOCH},
};

use super::{attach::attach_interactive, ipc_client::send_control_request};
use crate::{
    ScreenArgs,
    ipc::{ScreenAttachMode, ScreenIpcEndpoint, ScreenIpcRequest},
    sessions::find_builtin_screen_session_for_attach,
};

pub(crate) fn request_screen_attach(args: &ScreenArgs) -> io::Result<()> {
    let (mode, target) = match (&args.resume, &args.multi_attach) {
        (Some(target), None) => (ScreenAttachMode::Resume, target.as_deref()),
        (None, Some(target)) => (ScreenAttachMode::MultiAttach, target.as_deref()),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                terman_common::builtin_screen_attach_target_required_hint(),
            ));
        }
    };

    let session = find_builtin_screen_session_for_attach(target)?;
    let endpoint = session
        .ipc_endpoint
        .as_deref()
        .map(ScreenIpcEndpoint::from_raw_name)
        .unwrap_or_else(|| ScreenIpcEndpoint::for_session(&session.name));
    let session_name = session.name.clone();
    let client_id = new_attach_client_id();
    let mut stream = endpoint.connect_options()?.connect_sync()?;
    let request = ScreenIpcRequest::Attach {
        mode,
        target: Some(session_name.clone()),
        detach_existing: args.detach_existing,
        client_id: Some(client_id.clone()),
    };

    serde_json::to_writer(&mut stream, &request)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    attach_interactive(endpoint, stream, client_id, session_name)
}

pub(crate) fn request_screen_server_ready(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    send_control_request(endpoint, ScreenIpcRequest::Ping)
}

fn new_attach_client_id() -> String {
    let entropy = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    format!("{:x}-{entropy:x}", process::id())
}