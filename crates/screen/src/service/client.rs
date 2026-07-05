use std::io::{self, Write};

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
    let mut stream = endpoint.connect_options()?.connect_sync()?;
    let request = ScreenIpcRequest::Attach {
        mode,
        target: Some(session.name),
        detach_existing: args.detach_existing,
    };

    serde_json::to_writer(&mut stream, &request)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    attach_interactive(endpoint, stream)
}

pub(crate) fn request_screen_server_ready(session_name: &str) -> io::Result<()> {
    let endpoint = ScreenIpcEndpoint::for_session(session_name);
    send_control_request(&endpoint, ScreenIpcRequest::Ping)
}
