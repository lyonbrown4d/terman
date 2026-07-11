use std::{error::Error, io};

use crate::{
    args::{
        target_session_name_arg, window_option_name_arg, window_option_value_arg,
    },
    ipc::{TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
    sessions::load_builtin_tmux_sessions,
    window_command_support::{
        resolve_target_window_index, session_endpoint, session_not_found_error,
        unexpected_response_error,
    },
};

pub(crate) fn set_builtin_tmux_window_option(
    args: &[String],
) -> Result<(), Box<dyn Error>> {
    let option = window_option_name_arg(args).ok_or_else(option_value_error)?;
    if option != "synchronize-panes" {
        return Err(unsupported_option_error(option.as_str()));
    }
    let enabled = synchronize_panes_value(window_option_value_arg(args).as_deref())?;
    let target = target_session_name_arg(args)
        .ok_or_else(|| Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_tmux_target_required_hint(),
        )))?;
    let Some(session) = load_builtin_tmux_sessions()?
        .into_iter()
        .find(|session| session.name == target)
    else {
        return Err(session_not_found_error(&target));
    };
    let window = resolve_target_window_index(&session, args)?;
    match request_endpoint_response(
        &session_endpoint(&session),
        TmuxIpcRequest::SetSynchronizePanes {
            window: Some(window),
            enabled,
        },
    )? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => {
            Err(Box::new(io::Error::new(io::ErrorKind::Unsupported, reason)))
        }
        response => Err(unexpected_response_error(response)),
    }
}

fn synchronize_panes_value(value: Option<&str>) -> Result<Option<bool>, Box<dyn Error>> {
    match value.map(str::to_ascii_lowercase).as_deref() {
        None | Some("toggle") => Ok(None),
        Some("on" | "1" | "true") => Ok(Some(true)),
        Some("off" | "0" | "false") => Ok(Some(false)),
        Some(_) => Err(option_value_error()),
    }
}

fn unsupported_option_error(option: &str) -> Box<dyn Error> {
    Box::new(io::Error::new(
        io::ErrorKind::Unsupported,
        terman_common::builtin_tmux_window_option_unsupported_hint(option),
    ))
}

fn option_value_error() -> Box<dyn Error> {
    Box::new(io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_tmux_synchronize_panes_required_hint(),
    ))
}