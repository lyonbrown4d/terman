use super::{MessageKey, localized_message};

pub fn builtin_tmux_no_sessions_hint() -> String {
    localized_message(MessageKey::BuiltinTmuxNoSessions, &[])
}

pub fn builtin_tmux_cli_about() -> String {
    localized_message(MessageKey::BuiltinTmuxCliAbout, &[])
}

pub fn builtin_tmux_cli_examples() -> String {
    localized_message(MessageKey::BuiltinTmuxCliExamples, &[])
}

pub fn builtin_tmux_attach_help() -> String {
    localized_message(MessageKey::BuiltinTmuxAttachHelp, &[])
}
pub fn builtin_tmux_prefix_status_hint() -> String {
    localized_message(MessageKey::BuiltinTmuxPrefixStatus, &[])
}

pub fn builtin_tmux_session_list_entry_hint(
    name: &str,
    windows: u32,
    attached_clients: u32,
) -> String {
    let windows = windows.to_string();
    let attached_clients = attached_clients.to_string();
    localized_message(
        MessageKey::BuiltinTmuxSessionListEntry,
        &[
            ("name", name),
            ("windows", &windows),
            ("attached_clients", &attached_clients),
        ],
    )
}

pub fn builtin_tmux_client_list_entry_hint(session: &str, attached_clients: u32) -> String {
    let attached_clients = attached_clients.to_string();
    localized_message(
        MessageKey::BuiltinTmuxClientListEntry,
        &[
            ("session", session),
            ("attached_clients", &attached_clients),
        ],
    )
}

pub fn builtin_tmux_window_list_entry_hint(session: &str, index: u32, name: &str) -> String {
    let index = index.to_string();
    localized_message(
        MessageKey::BuiltinTmuxWindowListEntry,
        &[("session", session), ("index", &index), ("name", name)],
    )
}

pub fn builtin_tmux_session_created_hint(name: &str) -> String {
    localized_message(MessageKey::BuiltinTmuxSessionCreated, &[("name", name)])
}

pub fn builtin_tmux_window_created_hint(session: &str, windows: u32) -> String {
    let windows = windows.to_string();
    localized_message(
        MessageKey::BuiltinTmuxWindowCreated,
        &[("session", session), ("windows", &windows)],
    )
}

pub fn builtin_tmux_window_killed_hint(session: &str, windows: u32) -> String {
    let windows = windows.to_string();
    localized_message(
        MessageKey::BuiltinTmuxWindowKilled,
        &[("session", session), ("windows", &windows)],
    )
}

pub fn builtin_tmux_window_name_required_hint() -> String {
    localized_message(MessageKey::BuiltinTmuxWindowNameRequired, &[])
}

pub fn builtin_tmux_window_not_found_hint(session: &str, index: impl ToString) -> String {
    let index = index.to_string();
    localized_message(
        MessageKey::BuiltinTmuxWindowNotFound,
        &[("session", session), ("index", &index)],
    )
}

pub fn builtin_tmux_session_exists_hint(name: &str) -> String {
    localized_message(MessageKey::BuiltinTmuxSessionExists, &[("name", name)])
}

pub fn builtin_tmux_session_name_required_hint() -> String {
    localized_message(MessageKey::BuiltinTmuxSessionNameRequired, &[])
}

pub fn builtin_tmux_session_killed_hint(name: &str) -> String {
    localized_message(MessageKey::BuiltinTmuxSessionKilled, &[("name", name)])
}

pub fn builtin_tmux_session_not_found_hint(name: &str) -> String {
    localized_message(MessageKey::BuiltinTmuxSessionNotFound, &[("name", name)])
}

pub fn builtin_tmux_target_required_hint() -> String {
    localized_message(MessageKey::BuiltinTmuxTargetRequired, &[])
}

pub fn builtin_tmux_internal_server_session_required_hint() -> String {
    localized_message(MessageKey::BuiltinTmuxInternalServerSessionRequired, &[])
}

pub fn builtin_tmux_internal_server_exited_hint(code: i32) -> String {
    let code = code.to_string();
    localized_message(
        MessageKey::BuiltinTmuxInternalServerExited,
        &[("code", &code)],
    )
}

pub fn builtin_tmux_server_not_responding_hint() -> String {
    localized_message(MessageKey::BuiltinTmuxServerNotResponding, &[])
}

pub fn builtin_tmux_server_not_ready_hint() -> String {
    localized_message(MessageKey::BuiltinTmuxServerNotReady, &[])
}

pub fn builtin_tmux_unexpected_info_response_hint(response: &str) -> String {
    localized_message(
        MessageKey::BuiltinTmuxUnexpectedInfoResponse,
        &[("response", response)],
    )
}

pub fn builtin_tmux_unexpected_response_hint(response: &str) -> String {
    localized_message(
        MessageKey::BuiltinTmuxUnexpectedResponse,
        &[("response", response)],
    )
}

pub fn builtin_tmux_message_required_hint() -> String {
    localized_message(MessageKey::BuiltinTmuxMessageRequired, &[])
}

pub fn builtin_tmux_keys_required_hint() -> String {
    localized_message(MessageKey::BuiltinTmuxKeysRequired, &[])
}

pub fn builtin_tmux_command_unsupported_hint(command: &str) -> String {
    localized_message(
        MessageKey::BuiltinTmuxCommandUnsupported,
        &[("command", command)],
    )
}
pub fn builtin_tmux_attach_window_list(windows: &str) -> String {
    localized_message(MessageKey::BuiltinTmuxAttachWindowList, &[("windows", windows)])
}
pub fn builtin_tmux_pane_list_entry_hint(
    session: &str,
    window: u32,
    pane: u32,
    name: &str,
    active: bool,
) -> String {
    let window = window.to_string();
    let pane = pane.to_string();
    let active = active.to_string();
    localized_message(
        MessageKey::BuiltinTmuxPaneListEntry,
        &[("session", session), ("window", &window), ("pane", &pane), ("name", name), ("active", &active)],
    )
}
pub fn builtin_tmux_pane_not_found_hint(session: &str, window: u32, pane: u32) -> String {
    let window = window.to_string();
    let pane = pane.to_string();
    localized_message(
        MessageKey::BuiltinTmuxPaneNotFound,
        &[("session", session), ("window", &window), ("pane", &pane)],
    )
}
pub fn builtin_tmux_pane_size_required_hint() -> String {
    localized_message(MessageKey::BuiltinTmuxPaneSizeRequired, &[])
}

pub fn builtin_tmux_copy_status_hint(
    line: usize,
    total: usize,
    selecting: bool,
) -> String {
    let line = line.to_string();
    let total = total.to_string();
    localized_message(
        if selecting {
            MessageKey::BuiltinTmuxCopySelectionStatus
        } else {
            MessageKey::BuiltinTmuxCopyStatus
        },
        &[("line", &line), ("total", &total)],
    )
}