use super::{MessageKey, localized_message};

pub fn builtin_tmux_no_sessions_hint() -> String {
    localized_message(MessageKey::BuiltinTmuxNoSessions, &[])
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

pub fn builtin_tmux_window_list_entry_hint(session: &str, index: u32) -> String {
    let index = index.to_string();
    localized_message(
        MessageKey::BuiltinTmuxWindowListEntry,
        &[("session", session), ("index", &index)],
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

pub fn builtin_tmux_command_unsupported_hint(command: &str) -> String {
    localized_message(
        MessageKey::BuiltinTmuxCommandUnsupported,
        &[("command", command)],
    )
}
