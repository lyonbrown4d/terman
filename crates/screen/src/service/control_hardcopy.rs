use std::{
    fs, io,
    path::{Path, PathBuf},
};

use super::{
    control_parse::control_command_payload,
    control_session::{request_session_response, send_session_control_request},
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
        PathBuf::from(path.trim())
    };
    request_session_hardcopy(args, &path)
}

pub(super) fn request_hardcopydir_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let path = hardcopydir_path(&payload)?;
    send_session_control_request(args, ScreenIpcRequest::SetHardcopyDir { path })
}

fn hardcopydir_path(payload: &str) -> io::Result<PathBuf> {
    let path = payload.trim();
    if path.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_hardcopydir_required_hint(),
        ));
    }
    let path = fs::canonicalize(path)?;
    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_hardcopydir_required_hint(),
        ));
    }
    Ok(path)
}

fn default_hardcopy_path(args: &ScreenArgs) -> io::Result<PathBuf> {
    match request_session_response(args, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            active_window,
            hardcopy_dir,
            windows,
            ..
        } => Ok(numbered_hardcopy_path(
            hardcopy_dir.as_deref(),
            selected_index(args, active_window, &windows)?,
        )),
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(unexpected_response_error(&response)),
    }
}

fn numbered_hardcopy_path(hardcopy_dir: Option<&Path>, index: usize) -> PathBuf {
    let file_name = format!("hardcopy.{index}");
    hardcopy_dir
        .map(|directory| directory.join(&file_name))
        .unwrap_or_else(|| PathBuf::from(file_name))
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

fn request_session_hardcopy(args: &ScreenArgs, path: &Path) -> io::Result<()> {
    match request_with_window_target(args, ScreenIpcRequest::Hardcopy, request_session_response)? {
        ScreenIpcResponse::Hardcopy { bytes } => {
            fs::write(path, &bytes)?;
            let path = path.display().to_string();
            println!(
                "{}",
                terman_common::builtin_screen_control_hardcopy_complete_hint(&path, bytes.len())
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
    use std::path::{Path, PathBuf};

    use super::{numbered_hardcopy_path, selected_index};
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

    #[test]
    fn joins_hardcopydir_with_default_file_name() {
        assert_eq!(
            numbered_hardcopy_path(None, 3),
            PathBuf::from("hardcopy.3")
        );
        assert_eq!(
            numbered_hardcopy_path(Some(Path::new("copies")), 3),
            PathBuf::from("copies").join("hardcopy.3")
        );
    }
}