use std::{error::Error, io, io::Write};

use crate::{
    args::{target_pane_index_arg, target_session_name_arg, target_window_index_arg},
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
    sessions::{BuiltinTmuxSession, load_builtin_tmux_sessions},
};

pub(crate) fn capture_builtin_tmux_pane(args: &[String]) -> Result<(), Box<dyn Error>> {
    let target = target_session_name_arg(args).ok_or_else(target_required_error)?;
    let Some(session) = load_builtin_tmux_sessions()?
        .into_iter()
        .find(|session| session.name == target)
    else {
        return Err(session_not_found_error(&target));
    };
    let request = TmuxIpcRequest::CapturePane {
        window: target_window_index_arg(args).map(|index| index as u32),
        pane: target_pane_index_arg(args).map(|index| index as u32),
    };
    match request_endpoint_response(&session_endpoint(&session), request)? {
        TmuxIpcResponse::Captured { bytes } => {
            let mut stdout = io::stdout();
            stdout.write_all(&bytes)?;
            stdout.flush()?;
            Ok(())
        }
        TmuxIpcResponse::Rejected { reason } => {
            Err(Box::new(io::Error::new(io::ErrorKind::Unsupported, reason)))
        }
        response => Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")),
        ))),
    }
}

fn session_endpoint(session: &BuiltinTmuxSession) -> TmuxIpcEndpoint {
    session
        .ipc_endpoint
        .as_deref()
        .map(TmuxIpcEndpoint::from_raw_name)
        .unwrap_or_else(|| TmuxIpcEndpoint::for_session(&session.name))
}

fn target_required_error() -> Box<dyn Error> {
    Box::new(io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_tmux_target_required_hint(),
    ))
}

fn session_not_found_error(target: &str) -> Box<dyn Error> {
    Box::new(io::Error::new(
        io::ErrorKind::NotFound,
        terman_common::builtin_tmux_session_not_found_hint(target),
    ))
}
