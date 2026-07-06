use std::io;

use crate::{
    ScreenArgs,
    ipc::{ScreenIpcRequest, ScreenIpcResponse},
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
        } => {
            let index = parse_window_selector(selector, active_window)?;
            if !windows.iter().any(|window| window.index == index) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    terman_common::builtin_screen_control_select_unsupported_hint(selector),
                ));
            }
            Ok(Some(WindowTarget {
                index,
                restore: active_window,
            }))
        }
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

fn parse_window_selector(selector: &str, active_window: usize) -> io::Result<usize> {
    match selector {
        "." | "#" => Ok(active_window),
        value => value.parse::<usize>().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                terman_common::builtin_screen_control_select_unsupported_hint(selector),
            )
        }),
    }
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

fn unexpected_response_error(response: &ScreenIpcResponse) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        terman_common::builtin_screen_control_unexpected_response_hint(&format!("{response:?}")),
    )
}