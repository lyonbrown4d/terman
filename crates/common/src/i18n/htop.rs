use crate::i18n::{MessageKey, localized_message};

pub fn builtin_htop_cli_about() -> String {
    localized_message(MessageKey::BuiltinHtopCliAbout, &[])
}

pub fn builtin_htop_tab_overview_hint() -> String {
    localized_message(MessageKey::BuiltinHtopTabOverview, &[])
}

pub fn builtin_htop_tab_processes_hint() -> String {
    localized_message(MessageKey::BuiltinHtopTabProcesses, &[])
}

pub fn builtin_htop_tab_io_hint() -> String {
    localized_message(MessageKey::BuiltinHtopTabIo, &[])
}

pub fn builtin_htop_tab_network_hint() -> String {
    localized_message(MessageKey::BuiltinHtopTabNetwork, &[])
}

pub fn builtin_htop_help_hint() -> String {
    localized_message(MessageKey::BuiltinHtopHelp, &[])
}
