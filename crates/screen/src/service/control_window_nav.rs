use std::io;

use crate::{
    ScreenArgs,
    ipc::{ScreenIpcRequest, ScreenIpcResponse},
    region_types::{ScreenRegionAxis, ScreenRegionFocus},
};

type SessionRequester = fn(&ScreenArgs, ScreenIpcRequest) -> io::Result<ScreenIpcResponse>;

pub(super) fn request_window_navigation_command(
    args: &ScreenArgs,
    command: &str,
    request: SessionRequester,
) -> io::Result<()> {
    let request_kind = match command {
        "next" => ScreenIpcRequest::NextWindow,
        "prev" | "previous" => ScreenIpcRequest::PreviousWindow,
        "other" => ScreenIpcRequest::LastWindow,
        "split" => ScreenIpcRequest::SplitRegion {
            axis: split_axis(args),
        },
        "focus" => ScreenIpcRequest::FocusRegion {
            target: focus_target(args),
        },
        "remove" => ScreenIpcRequest::RemoveRegion,
        "only" => ScreenIpcRequest::OnlyRegion,
        _ => return Ok(()),
    };
    match request(args, request_kind)? {
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

fn split_axis(args: &ScreenArgs) -> ScreenRegionAxis {
    if execute_words(args).any(|word| matches!(word, "-v" | "vertical")) {
        ScreenRegionAxis::Vertical
    } else {
        ScreenRegionAxis::Horizontal
    }
}

fn focus_target(args: &ScreenArgs) -> ScreenRegionFocus {
    match execute_words(args).skip(1).next() {
        Some("prev" | "previous" | "up" | "left") => ScreenRegionFocus::Previous,
        Some("top" | "first") => ScreenRegionFocus::First,
        Some("bottom" | "last") => ScreenRegionFocus::Last,
        _ => ScreenRegionFocus::Next,
    }
}

fn execute_words(args: &ScreenArgs) -> impl Iterator<Item = &str> {
    args.execute.as_deref().unwrap_or_default().split_whitespace()
}