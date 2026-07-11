use std::{error::Error, io};

use crate::{
    args::{
        target_pane_index_arg, target_session_name_arg, target_window_index_arg,
    },
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
    sessions::{BuiltinTmuxSession, load_builtin_tmux_sessions},
};

struct PaneInfo {
    window_index: u32,
    pane_indexes: Vec<u32>,
}

pub(crate) fn swap_builtin_tmux_pane(args: &[String]) -> Result<(), Box<dyn Error>> {
    let (_, session) = target_session(args)?;
    let info = query_pane_info(
        &session,
        target_window_index_arg(args).map(|index| index as u32),
    )?;
    let direction = swap_direction(args);
    let destination = target_pane_index_arg(args).map(|index| index as u32);
    let source = if direction.is_some() {
        source_pane_index_arg(args).or(destination)
    } else {
        source_pane_index_arg(args)
    };
    let target_pane = if direction.is_some() { None } else { destination };
    if let Some(source) = source {
        require_pane(&info, source)?;
    }
    if let Some(target_pane) = target_pane {
        require_pane(&info, target_pane)?;
    }
    request_accepted(
        &session,
        TmuxIpcRequest::SwapPane {
            window: Some(info.window_index),
            source,
            target: target_pane,
            forward: direction.unwrap_or(true),
        },
    )
}

fn swap_direction(args: &[String]) -> Option<bool> {
    args.iter().fold(None, |direction, arg| match arg.as_str() {
        "-U" | "--up" => Some(false),
        "-D" | "--down" => Some(true),
        _ => direction,
    })
}

fn source_pane_index_arg(args: &[String]) -> Option<u32> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        let value = if arg == "-s" || arg == "--source-pane" {
            iter.next()
        } else if let Some(value) = arg.strip_prefix("-s").filter(|value| !value.is_empty()) {
            return parse_pane_index(value);
        } else if let Some(value) = arg.strip_prefix("--source-pane=") {
            return parse_pane_index(value);
        } else {
            None
        };
        if let Some(value) = value {
            return parse_pane_index(value);
        }
    }
    None
}

fn parse_pane_index(target: &str) -> Option<u32> {
    target
        .rsplit_once('.')
        .map(|(_, pane)| pane)
        .unwrap_or(target)
        .parse()
        .ok()
}

fn target_session(args: &[String]) -> Result<(String, BuiltinTmuxSession), Box<dyn Error>> {
    let target = target_session_name_arg(args).ok_or_else(target_required_error)?;
    let session = load_builtin_tmux_sessions()?
        .into_iter()
        .find(|session| session.name == target)
        .ok_or_else(|| session_not_found_error(&target))?;
    Ok((target, session))
}

fn query_pane_info(
    session: &BuiltinTmuxSession,
    window: Option<u32>,
) -> Result<PaneInfo, Box<dyn Error>> {
    match request_endpoint_response(&session_endpoint(session), TmuxIpcRequest::PaneInfo { window })?
    {
        TmuxIpcResponse::Panes {
            window_index,
            pane_indexes,
            ..
        } => Ok(PaneInfo {
            window_index,
            pane_indexes,
        }),
        TmuxIpcResponse::Rejected { reason } => {
            Err(Box::new(io::Error::new(io::ErrorKind::Unsupported, reason)))
        }
        response => Err(unexpected_response_error(response)),
    }
}

fn request_accepted(
    session: &BuiltinTmuxSession,
    request: TmuxIpcRequest,
) -> Result<(), Box<dyn Error>> {
    match request_endpoint_response(&session_endpoint(session), request)? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => {
            Err(Box::new(io::Error::new(io::ErrorKind::Unsupported, reason)))
        }
        response => Err(unexpected_response_error(response)),
    }
}

fn require_pane(info: &PaneInfo, pane: u32) -> Result<(), Box<dyn Error>> {
    if info.pane_indexes.contains(&pane) {
        Ok(())
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::NotFound,
            terman_common::builtin_tmux_pane_not_found_hint(
                "current",
                info.window_index,
                pane,
            ),
        )))
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

fn unexpected_response_error(response: TmuxIpcResponse) -> Box<dyn Error> {
    Box::new(io::Error::new(
        io::ErrorKind::InvalidData,
        terman_common::builtin_tmux_unexpected_info_response_hint(&format!("{response:?}")),
    ))
}