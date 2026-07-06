use crate::i18n::{MessageKey, localized_message};

pub fn builtin_screen_control_info_hint(
    session_name: &str,
    replay_bytes: usize,
    attach_clients: usize,
    cols: Option<u16>,
    rows: Option<u16>,
    scrollback_lines: usize,
) -> String {
    let replay_bytes = replay_bytes.to_string();
    let attach_clients = attach_clients.to_string();
    let scrollback_lines = scrollback_lines.to_string();
    let cols = cols
        .map(|value| value.to_string())
        .unwrap_or_else(|| String::from("?"));
    let rows = rows
        .map(|value| value.to_string())
        .unwrap_or_else(|| String::from("?"));
    localized_message(
        MessageKey::BuiltinScreenControlInfo,
        &[
            ("session_name", session_name),
            ("replay_bytes", &replay_bytes),
            ("attach_clients", &attach_clients),
            ("cols", &cols),
            ("rows", &rows),
            ("scrollback_lines", &scrollback_lines),
        ],
    )
}

pub fn builtin_screen_control_dinfo_hint(
    session_name: &str,
    attach_clients: usize,
    cols: Option<u16>,
    rows: Option<u16>,
    active_window: usize,
    term: &str,
) -> String {
    let attach_clients = attach_clients.to_string();
    let active_window = active_window.to_string();
    let cols = cols
        .map(|value| value.to_string())
        .unwrap_or_else(|| String::from("?"));
    let rows = rows
        .map(|value| value.to_string())
        .unwrap_or_else(|| String::from("?"));
    localized_message(
        MessageKey::BuiltinScreenControlDinfo,
        &[
            ("session_name", session_name),
            ("attach_clients", &attach_clients),
            ("cols", &cols),
            ("rows", &rows),
            ("active_window", &active_window),
            ("term", term),
        ],
    )
}

pub fn builtin_screen_control_displays_entry_hint(
    session_name: &str,
    attach_clients: usize,
    cols: Option<u16>,
    rows: Option<u16>,
) -> String {
    let attach_clients = attach_clients.to_string();
    let cols = cols
        .map(|value| value.to_string())
        .unwrap_or_else(|| String::from("?"));
    let rows = rows
        .map(|value| value.to_string())
        .unwrap_or_else(|| String::from("?"));
    localized_message(
        MessageKey::BuiltinScreenControlDisplaysEntry,
        &[
            ("session_name", session_name),
            ("attach_clients", &attach_clients),
            ("cols", &cols),
            ("rows", &rows),
        ],
    )
}

pub fn builtin_screen_control_windows_entry_hint(
    index: usize,
    active: bool,
    title: &str,
    replay_bytes: usize,
    attach_clients: usize,
    cols: Option<u16>,
    rows: Option<u16>,
) -> String {
    let index = index.to_string();
    let active_marker = if active { "*" } else { " " };
    let replay_bytes = replay_bytes.to_string();
    let attach_clients = attach_clients.to_string();
    let cols = cols
        .map(|value| value.to_string())
        .unwrap_or_else(|| String::from("?"));
    let rows = rows
        .map(|value| value.to_string())
        .unwrap_or_else(|| String::from("?"));
    localized_message(
        MessageKey::BuiltinScreenControlWindowsEntry,
        &[
            ("index", &index),
            ("active_marker", active_marker),
            ("title", title),
            ("replay_bytes", &replay_bytes),
            ("attach_clients", &attach_clients),
            ("cols", &cols),
            ("rows", &rows),
        ],
    )
}

pub fn builtin_screen_control_unexpected_response_hint(response: &str) -> String {
    localized_message(
        MessageKey::BuiltinScreenControlUnexpectedResponse,
        &[("response", response)],
    )
}