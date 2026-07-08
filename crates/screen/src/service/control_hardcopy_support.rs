use std::{
    fs,
    fs::OpenOptions,
    io::{self, Write},
    path::{Path, PathBuf},
};

use super::{
    control_session::request_session_response,
    control_target::resolve_window_selector,
};
use crate::{
    ScreenArgs,
    ipc::{ScreenIpcRequest, ScreenIpcResponse, ScreenWindowInfo},
};

pub(super) struct HardcopyOptions {
    pub(super) path: PathBuf,
    pub(super) append: bool,
    pub(super) include_history: bool,
}

struct HardcopyPayload {
    path: Option<PathBuf>,
    include_history: bool,
}

pub(super) fn parse_hardcopy_append(payload: &str) -> io::Result<bool> {
    let mut parts = payload.split_whitespace();
    let Some(state) = parts.next() else { return Err(hardcopy_append_required_error()); };
    if parts.next().is_some() { return Err(hardcopy_append_required_error()); }
    match state.to_ascii_lowercase().as_str() {
        "on" | "1" | "true" => Ok(true),
        "off" | "0" | "false" => Ok(false),
        _ => Err(hardcopy_append_required_error()),
    }
}

pub(super) fn hardcopydir_path(payload: &str) -> io::Result<PathBuf> {
    let path = payload.trim();
    if path.is_empty() { return Err(hardcopydir_required_error()); }
    let path = fs::canonicalize(path)?;
    if !path.is_dir() { return Err(hardcopydir_required_error()); }
    Ok(path)
}

pub(super) fn hardcopy_options(args: &ScreenArgs, payload: &str) -> io::Result<HardcopyOptions> {
    match request_session_response(args, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info { active_window, hardcopy_append, hardcopy_dir, windows, .. } => {
            let payload = parse_hardcopy_payload(payload);
            let path = match payload.path {
                Some(path) => path,
                None => numbered_hardcopy_path(
                    hardcopy_dir.as_deref(),
                    selected_index(args, active_window, &windows)?,
                ),
            };
            Ok(HardcopyOptions { path, append: hardcopy_append, include_history: payload.include_history })
        }
        ScreenIpcResponse::Rejected { reason } => Err(io::Error::new(io::ErrorKind::Unsupported, reason)),
        response => Err(unexpected_response_error(&response)),
    }
}

pub(super) fn write_hardcopy(path: &Path, append: bool, bytes: &[u8]) -> io::Result<()> {
    let mut options = OpenOptions::new();
    options.create(true).write(true);
    if append {
        options.append(true);
    } else {
        options.truncate(true);
    }
    let mut file = options.open(path)?;
    file.write_all(bytes)
}

pub(super) fn unexpected_response_error(response: &ScreenIpcResponse) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        terman_common::builtin_screen_control_unexpected_response_hint(&format!("{response:?}")),
    )
}

fn hardcopy_append_required_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_hardcopy_append_required_hint(),
    )
}

fn hardcopydir_required_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_hardcopydir_required_hint(),
    )
}

fn parse_hardcopy_payload(payload: &str) -> HardcopyPayload {
    let payload = payload.trim();
    if payload.is_empty() {
        return HardcopyPayload { path: None, include_history: false };
    }
    if payload == "-h" {
        return HardcopyPayload { path: None, include_history: true };
    }
    if let Some(rest) = payload.strip_prefix("-h") {
        if rest.chars().next().is_some_and(char::is_whitespace) {
            let path = rest.trim();
            return HardcopyPayload {
                path: (!path.is_empty()).then(|| PathBuf::from(path)),
                include_history: true,
            };
        }
    }
    HardcopyPayload { path: Some(PathBuf::from(payload)), include_history: false }
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
        Some(selector) if !selector.is_empty() => resolve_window_selector(selector, active_window, windows),
        _ => Ok(active_window),
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{numbered_hardcopy_path, parse_hardcopy_append, parse_hardcopy_payload, selected_index};
    use crate::{ScreenArgs, ipc::ScreenWindowInfo};

    fn windows() -> Vec<ScreenWindowInfo> {
        vec![
            ScreenWindowInfo { index: 0, title: String::from("shell"), active: true, replay_bytes: 1 },
            ScreenWindowInfo { index: 2, title: String::from("editor"), active: false, replay_bytes: 1 },
        ]
    }

    #[test]
    fn parses_hardcopy_append_state() {
        assert!(parse_hardcopy_append("on").unwrap());
        assert!(!parse_hardcopy_append("off").unwrap());
        assert!(parse_hardcopy_append("").is_err());
    }

    #[test]
    fn parses_hardcopy_h_option_without_treating_it_as_path() {
        let empty = parse_hardcopy_payload("");
        assert_eq!(empty.path, None);
        assert!(!empty.include_history);
        let history = parse_hardcopy_payload("-h copy.txt");
        assert_eq!(history.path, Some(PathBuf::from("copy.txt")));
        assert!(history.include_history);
        let path = parse_hardcopy_payload("-hardcopy");
        assert_eq!(path.path, Some(PathBuf::from("-hardcopy")));
        assert!(!path.include_history);
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
        assert_eq!(numbered_hardcopy_path(None, 3), PathBuf::from("hardcopy.3"));
        assert_eq!(
            numbered_hardcopy_path(Some(Path::new("copies")), 3),
            PathBuf::from("copies").join("hardcopy.3")
        );
    }
}