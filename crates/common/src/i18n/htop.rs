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

pub fn builtin_htop_help_panel_hint() -> String {
    localized_message(MessageKey::BuiltinHtopHelpPanel, &[])
}

pub fn builtin_htop_sort_menu_title_hint() -> String {
    localized_message(MessageKey::BuiltinHtopSortMenuTitle, &[])
}

pub fn builtin_htop_sort_menu_help_hint() -> String {
    localized_message(MessageKey::BuiltinHtopSortMenuHelp, &[])
}
pub fn builtin_htop_signal_menu_title_hint(pid: &str) -> String {
    localized_message(MessageKey::BuiltinHtopSignalMenuTitle, &[("pid", pid)])
}

pub fn builtin_htop_signal_menu_help_hint() -> String {
    localized_message(MessageKey::BuiltinHtopSignalMenuHelp, &[])
}

pub fn builtin_htop_signal_unsupported_hint() -> String {
    localized_message(MessageKey::BuiltinHtopSignalUnsupported, &[])
}

pub fn builtin_htop_signal_footer_hint(pid: &str) -> String {
    localized_message(MessageKey::BuiltinHtopSignalFooter, &[("pid", pid)])
}

pub fn builtin_htop_follow_status_hint(pid: &str) -> String {
    localized_message(MessageKey::BuiltinHtopFollowStatus, &[("pid", pid)])
}
