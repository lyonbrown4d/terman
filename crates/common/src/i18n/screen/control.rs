use crate::i18n::{MessageKey, localized_message};

pub fn builtin_screen_control_command_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlCommandRequired, &[])
}

pub fn builtin_screen_control_command_unsupported_hint(command: &str) -> String {
    localized_message(
        MessageKey::BuiltinScreenControlCommandUnsupported,
        &[("command", command)],
    )
}

pub fn builtin_screen_control_echo_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlEchoRequired, &[])
}

pub fn builtin_screen_control_stuff_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlStuffRequired, &[])
}

pub fn builtin_screen_control_resize_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlResizeRequired, &[])
}

pub fn builtin_screen_control_help_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlHelp, &[])
}

pub fn builtin_screen_control_select_unsupported_hint(selector: &str) -> String {
    localized_message(
        MessageKey::BuiltinScreenControlSelectUnsupported,
        &[("selector", selector)],
    )
}

pub fn builtin_screen_control_sleep_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlSleepRequired, &[])
}

pub fn builtin_screen_control_time_hint(unix_seconds: u64) -> String {
    let unix_seconds = unix_seconds.to_string();
    localized_message(
        MessageKey::BuiltinScreenControlTime,
        &[("unix_seconds", &unix_seconds)],
    )
}

pub fn builtin_screen_control_version_hint(version: &str) -> String {
    localized_message(MessageKey::BuiltinScreenControlVersion, &[("version", version)])
}

pub fn builtin_screen_control_info_hint(
    session_name: &str,
    replay_bytes: usize,
    attach_clients: usize,
    cols: Option<u16>,
    rows: Option<u16>,
) -> String {
    let replay_bytes = replay_bytes.to_string();
    let attach_clients = attach_clients.to_string();
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
    session_name: &str,
    replay_bytes: usize,
    attach_clients: usize,
    cols: Option<u16>,
    rows: Option<u16>,
) -> String {
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
            ("session_name", session_name),
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

pub fn builtin_screen_control_hardcopy_path_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlHardcopyPathRequired, &[])
}

pub fn builtin_screen_control_pastefile_path_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlPastefilePathRequired, &[])
}

pub fn builtin_screen_control_source_path_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlSourcePathRequired, &[])
}

pub fn builtin_screen_control_hardcopy_complete_hint(path: &str, bytes: usize) -> String {
    let bytes = bytes.to_string();
    localized_message(
        MessageKey::BuiltinScreenControlHardcopyComplete,
        &[("path", path), ("bytes", &bytes)],
    )
}