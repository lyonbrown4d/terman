use super::super::{MessageKey, localized_message};

pub fn builtin_screen_confirm_kill_hint() -> String {
    localized_message(MessageKey::BuiltinScreenConfirmKill, &[])
}

pub fn builtin_screen_confirm_quit_hint() -> String {
    localized_message(MessageKey::BuiltinScreenConfirmQuit, &[])
}