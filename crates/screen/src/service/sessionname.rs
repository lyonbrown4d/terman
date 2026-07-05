use std::io;

use super::{
    control_parse::control_command_payload,
    ipc_client::request_endpoint_response,
};
use crate::{
    ScreenArgs,
    ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse},
    sessions::{
        BuiltinScreenSession, RenameBuiltinScreenSession, find_builtin_screen_session_for_attach,
        rename_builtin_screen_session, validate_screen_session_name,
    },
};

pub(super) fn request_sessionname_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let new_name = control_command_payload(inline_payload, extra_args);
    validate_screen_session_name(&new_name)?;

    let session = find_builtin_screen_session_for_attach(args.session_name.as_deref())?;
    match rename_builtin_screen_session(&session.name, &new_name)? {
        RenameBuiltinScreenSession::Renamed => request_live_session_rename(&session, &new_name),
        RenameBuiltinScreenSession::SourceMissing => Err(io::Error::new(
            io::ErrorKind::NotFound,
            terman_common::builtin_screen_session_not_found_hint(&session.name),
        )),
        RenameBuiltinScreenSession::DestinationExists => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            terman_common::builtin_screen_session_exists_hint(&new_name),
        )),
    }
}

fn request_live_session_rename(session: &BuiltinScreenSession, name: &str) -> io::Result<()> {
    let endpoint = session_endpoint(session);
    match request_endpoint_response(
        &endpoint,
        ScreenIpcRequest::RenameSession {
            name: name.to_string(),
        },
    )? {
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

fn session_endpoint(session: &BuiltinScreenSession) -> ScreenIpcEndpoint {
    session
        .ipc_endpoint
        .as_deref()
        .map(ScreenIpcEndpoint::from_raw_name)
        .unwrap_or_else(|| ScreenIpcEndpoint::for_session(&session.name))
}
