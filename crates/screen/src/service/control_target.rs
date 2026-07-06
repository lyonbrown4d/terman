use std::io;

use crate::{
    ScreenArgs,
    ipc::{ScreenIpcRequest, ScreenIpcResponse, ScreenWindowInfo},
};

pub(super) type SessionRequester = fn(&ScreenArgs, ScreenIpcRequest) -> io::Result<ScreenIpcResponse>;

struct WindowTarget {
    index: usize,
    restore: usize,
}

pub(super) fn request_with_window_target(
    args: &ScreenArgs,
    request: ScreenIpcRequest,
    requester: SessionRequester,
) -> io::Result<ScreenIpcResponse> {
    let Some(target) = window_target(args, requester)? else {
        return requester(args, request);
    };
    if target.index == target.restore {
        return requester(args, request);
    }

    ensure_accepted(requester(
        args,
        ScreenIpcRequest::SelectWindow {
            index: target.index,
        },
    )?)?;
    let result = requester(args, request);
    let restore = requester(
        args,
        ScreenIpcRequest::SelectWindow {
            index: target.restore,
        },
    );

    match (result, restore) {
        (Err(err), _) => Err(err),
        (Ok(_), Err(err)) => Err(err),
        (Ok(response), Ok(restore_response)) => {
            ensure_accepted(restore_response)?;
            Ok(response)
        }
    }
}

fn window_target(
    args: &ScreenArgs,
    requester: SessionRequester,
) -> io::Result<Option<WindowTarget>> {
    let Some(selector) = args
        .window_selector
        .as_deref()
        .map(str::trim)
        .filter(|selector| !selector.is_empty())
    else {
        return Ok(None);
    };

    match requester(args, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            active_window,
            windows,
            ..
        } => Ok(Some(WindowTarget {
            index: resolve_window_selector(selector, active_window, &windows)?,
            restore: active_window,
        })),
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

pub(super) fn resolve_window_selector(
    selector: &str,
    active_window: usize,
    windows: &[ScreenWindowInfo],
) -> io::Result<usize> {
    match selector {
        "." | "#" => return Ok(active_window),
        _ => {}
    }

    if let Ok(index) = selector.parse::<usize>() {
        if windows.iter().any(|window| window.index == index) {
            return Ok(index);
        }
    }

    windows
        .iter()
        .find(|window| window.title == selector)
        .map(|window| window.index)
        .ok_or_else(|| unsupported_selector_error(selector))
}

fn ensure_accepted(response: ScreenIpcResponse) -> io::Result<()> {
    match response {
        ScreenIpcResponse::Accepted => Ok(()),
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

fn unsupported_selector_error(selector: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_select_unsupported_hint(selector),
    )
}

fn unexpected_response_error(response: &ScreenIpcResponse) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        terman_common::builtin_screen_control_unexpected_response_hint(&format!("{response:?}")),
    )
}