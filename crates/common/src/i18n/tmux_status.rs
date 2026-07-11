use super::{MessageKey, localized_message};

pub fn builtin_tmux_status_line_hint(session: &str, windows: &str) -> String {
    localized_message(MessageKey::BuiltinTmuxStatusLine, &[("session", session), ("windows", windows)])
}

pub fn builtin_tmux_kill_pane_confirm_hint() -> String {
    localized_message(MessageKey::BuiltinTmuxKillPaneConfirm, &[])
}

pub fn builtin_tmux_kill_window_confirm_hint() -> String {
    localized_message(MessageKey::BuiltinTmuxKillWindowConfirm, &[])
}