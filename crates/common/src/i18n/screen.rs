use super::{MessageKey, localized_message};

pub fn builtin_screen_no_sessions_hint() -> String {
    localized_message(MessageKey::BuiltinScreenNoSessions, &[])
}

pub fn builtin_screen_cli_about() -> String {
    localized_message(MessageKey::BuiltinScreenCliAbout, &[])
}

pub fn builtin_screen_cli_examples() -> String {
    localized_message(MessageKey::BuiltinScreenCliExamples, &[])
}

pub fn builtin_screen_session_list_header() -> String {
    localized_message(MessageKey::BuiltinScreenSessionListHeader, &[])
}

pub fn builtin_screen_session_list_entry_hint(
    name: &str,
    pid: &str,
    attach_clients: usize,
    replay_bytes: usize,
    cols: Option<u16>,
    rows: Option<u16>,
    cwd: &str,
    command: &str,
) -> String {
    let attach_clients = attach_clients.to_string();
    let replay_bytes = replay_bytes.to_string();
    let cols = cols
        .map(|value| value.to_string())
        .unwrap_or_else(|| String::from("?"));
    let rows = rows
        .map(|value| value.to_string())
        .unwrap_or_else(|| String::from("?"));
    localized_message(
        MessageKey::BuiltinScreenSessionListEntry,
        &[
            ("name", name),
            ("pid", pid),
            ("attach_clients", &attach_clients),
            ("replay_bytes", &replay_bytes),
            ("cols", &cols),
            ("rows", &rows),
            ("cwd", cwd),
            ("command", command),
        ],
    )
}

pub fn builtin_screen_session_exists_hint(name: &str) -> String {
    localized_message(MessageKey::BuiltinScreenSessionExists, &[("name", name)])
}

pub fn builtin_screen_session_name_empty_hint() -> String {
    localized_message(MessageKey::BuiltinScreenSessionNameEmpty, &[])
}

pub fn builtin_screen_session_record_invalid_hint() -> String {
    localized_message(MessageKey::BuiltinScreenSessionRecordInvalid, &[])
}

pub fn builtin_screen_unexpected_response_hint(response: &str) -> String {
    localized_message(
        MessageKey::BuiltinScreenUnexpectedResponse,
        &[("response", response)],
    )
}

pub fn builtin_screen_attach_unsupported_hint() -> String {
    localized_message(MessageKey::BuiltinScreenAttachUnsupported, &[])
}

pub fn builtin_screen_attach_help_hint() -> String {
    localized_message(MessageKey::BuiltinScreenAttachHelp, &[])
}

pub fn builtin_screen_attach_hardcopy_path_unavailable_hint() -> String {
    localized_message(MessageKey::BuiltinScreenAttachHardcopyPathUnavailable, &[])
}

pub fn builtin_screen_attach_target_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenAttachTargetRequired, &[])
}

pub fn builtin_screen_attach_output_thread_panicked_hint() -> String {
    localized_message(MessageKey::BuiltinScreenAttachOutputThreadPanicked, &[])
}

pub fn builtin_screen_session_not_found_hint(name: &str) -> String {
    localized_message(MessageKey::BuiltinScreenSessionNotFound, &[("name", name)])
}

pub fn builtin_screen_named_session_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenNamedSessionRequired, &[])
}

pub fn builtin_screen_server_timeout_hint() -> String {
    localized_message(MessageKey::BuiltinScreenServerTimeout, &[])
}

pub fn builtin_screen_service_timeout_hint() -> String {
    localized_message(MessageKey::BuiltinScreenServiceTimeout, &[])
}

pub fn builtin_screen_internal_server_session_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenInternalServerSessionRequired, &[])
}

pub fn builtin_screen_internal_server_exited_hint(code: i32) -> String {
    let code = code.to_string();
    localized_message(
        MessageKey::BuiltinScreenInternalServerExited,
        &[("code", &code)],
    )
}

pub fn builtin_screen_failure_hint(code: i32) -> String {
    let code = code.to_string();
    localized_message(MessageKey::BuiltinScreenFailure, &[("code", &code)])
}

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

pub fn builtin_screen_wipe_complete_hint(count: usize) -> String {
    let count = count.to_string();
    localized_message(MessageKey::BuiltinScreenWipeComplete, &[("count", &count)])
}
