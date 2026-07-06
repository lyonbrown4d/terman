use std::{fs, io};

use super::{
    control_parse::control_command_payload,
    control_session::request_session_response,
    control_target::{request_with_window_target, resolve_window_selector},
};
use crate::{
    ScreenArgs,
    ipc::{ScreenIpcRequest, ScreenIpcResponse, ScreenWindowInfo},
};

pub(super) fn request_hardcopy_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let path = control_command_payload(inline_payload, extra_args);
    let path = if path.trim().is_empty() {
        default_hardcopy_path(args)?
    } else {
        path
    };
    request_session_hardcopy(args, &path)
}

fn default_hardcopy_path(args: &ScreenArgs) -> io::Result<String> {
    match request_session_response(args, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            active_window,
            windows,
            ..
        } => Ok(format!("hardcopy.{}", selected_index(args, active_window, &windows)?)),
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

fn selected_index(
    args: &ScreenArgs,
    active_window: usize,
    windows: &[ScreenWindowInfo],
) -> io::Result<usize> {
    match args.window_selector.as_deref().map(str::trim) {
        Some(selector) if !selector.is_empty() => {
            resolve_window_selector(selector, active_window, windows)
        }
        _ => Ok(active_window),
    }
}

fn request_session_hardcopy(args: &ScreenArgs, path: &str) -> io::Result<()> {
    match request_with_window_target(args, ScreenIpcRequest::Hardcopy, request_session_response)? {
        ScreenIpcResponse::Hardcopy { bytes } => {
            fs::write(path, &bytes)?;
            println!(
                "{}",
                terman_common::builtin_screen_control_hardcopy_complete_hint(path, bytes.len())
            );
            Ok(())
        }
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

#[cfg(test)]
mod tests {
    use super::selected_index;
    use crate::{ScreenArgs, ipc::ScreenWindowInfo};

    fn windows() -> Vec<ScreenWindowInfo> {
        vec![
            ScreenWindowInfo {
                index: 0,
                title: String::from("shell"),
                active: true,
                replay_bytes: 1,
            },
            ScreenWindowInfo {
                index: 2,
                title: String::from("editor"),
                active: false,
                replay_bytes: 1,
            },
        ]
    }

    #[test]
    fn selects_active_or_target_window_for_default_path() {
        let args = ScreenArgs::default();
        assert_eq!(selected_index(&args, 0, &windows()).unwrap(), 0);

        let mut args = ScreenArgs::default();
        args.window_selector = Some(String::from("editor"));
        assert_eq!(selected_index(&args, 0, &windows()).unwrap(), 2);
    }
}