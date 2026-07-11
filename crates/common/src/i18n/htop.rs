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
pub fn builtin_htop_tree_collapse_hint() -> String {
    localized_message(MessageKey::BuiltinHtopTreeCollapse, &[])
}

pub fn builtin_htop_tree_expand_hint() -> String {
    localized_message(MessageKey::BuiltinHtopTreeExpand, &[])
}

pub fn builtin_htop_tree_toggle_all_hint() -> String {
    localized_message(MessageKey::BuiltinHtopTreeToggleAll, &[])
}

pub fn builtin_htop_sort_menu_title_hint() -> String {
    localized_message(MessageKey::BuiltinHtopSortMenuTitle, &[])
}

pub fn builtin_htop_sort_menu_help_hint() -> String {
    localized_message(MessageKey::BuiltinHtopSortMenuHelp, &[])
}

pub fn builtin_htop_user_filter_hint() -> String {
    localized_message(MessageKey::BuiltinHtopUserFilter, &[])
}

pub fn builtin_htop_all_users_hint() -> String {
    localized_message(MessageKey::BuiltinHtopAllUsers, &[])
}

pub fn builtin_htop_user_menu_title_hint() -> String {
    localized_message(MessageKey::BuiltinHtopUserMenuTitle, &[])
}

pub fn builtin_htop_user_menu_help_hint() -> String {
    localized_message(MessageKey::BuiltinHtopUserMenuHelp, &[])
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
pub fn builtin_htop_setup_title_hint() -> String {
    localized_message(MessageKey::BuiltinHtopSetupTitle, &[])
}
pub fn builtin_htop_setup_help_hint() -> String {
    localized_message(MessageKey::BuiltinHtopSetupHelp, &[])
}
pub fn builtin_htop_setup_refresh_hint() -> String {
    localized_message(MessageKey::BuiltinHtopSetupRefresh, &[])
}
pub fn builtin_htop_setup_tree_hint() -> String {
    localized_message(MessageKey::BuiltinHtopSetupTree, &[])
}
pub fn builtin_htop_setup_sort_direction_hint() -> String {
    localized_message(MessageKey::BuiltinHtopSetupSortDirection, &[])
}
pub fn builtin_htop_setup_toggle_hint(enabled: bool) -> String {
    let key = if enabled { MessageKey::BuiltinHtopSetupEnabled } else { MessageKey::BuiltinHtopSetupDisabled };
    localized_message(key, &[])
}
pub fn builtin_htop_setup_direction_hint(inverted: bool) -> String {
    let key = if inverted { MessageKey::BuiltinHtopSetupAscending } else { MessageKey::BuiltinHtopSetupDescending };
    localized_message(key, &[])
}

pub fn builtin_htop_footer_help_hint() -> String {
    localized_message(MessageKey::BuiltinHtopFooterHelp, &[])
}

pub fn builtin_htop_footer_search_hint() -> String {
    localized_message(MessageKey::BuiltinHtopFooterSearch, &[])
}

pub fn builtin_htop_footer_filter_hint() -> String {
    localized_message(MessageKey::BuiltinHtopFooterFilter, &[])
}

pub fn builtin_htop_footer_priority_higher_hint() -> String {
    localized_message(MessageKey::BuiltinHtopFooterPriorityHigher, &[])
}

pub fn builtin_htop_footer_priority_lower_hint() -> String {
    localized_message(MessageKey::BuiltinHtopFooterPriorityLower, &[])
}

pub fn builtin_htop_footer_kill_hint() -> String {
    localized_message(MessageKey::BuiltinHtopFooterKill, &[])
}

pub fn builtin_htop_footer_delay_hint() -> String {
    localized_message(MessageKey::BuiltinHtopFooterDelay, &[])
}

pub fn builtin_htop_footer_quit_hint() -> String {
    localized_message(MessageKey::BuiltinHtopFooterQuit, &[])
}

pub fn builtin_htop_footer_yes_hint() -> String {
    localized_message(MessageKey::BuiltinHtopFooterYes, &[])
}

pub fn builtin_htop_footer_no_hint() -> String {
    localized_message(MessageKey::BuiltinHtopFooterNo, &[])
}

pub fn builtin_htop_footer_search_prompt_hint() -> String {
    localized_message(MessageKey::BuiltinHtopFooterSearchPrompt, &[])
}

pub fn builtin_htop_footer_filter_prompt_hint() -> String {
    localized_message(MessageKey::BuiltinHtopFooterFilterPrompt, &[])
}

pub fn builtin_htop_footer_tree_prompt_hint() -> String {
    localized_message(MessageKey::BuiltinHtopFooterTreePrompt, &[])
}

pub fn builtin_htop_footer_select_prompt_hint() -> String {
    localized_message(MessageKey::BuiltinHtopFooterSelectPrompt, &[])
}

pub fn builtin_htop_footer_view_flat_hint() -> String {
    localized_message(MessageKey::BuiltinHtopFooterViewFlat, &[])
}

pub fn builtin_htop_tag() -> String {
    localized_message(MessageKey::BuiltinHtopTag, &[])
}

pub fn builtin_htop_untag_all() -> String {
    localized_message(MessageKey::BuiltinHtopUntagAll, &[])
}

pub fn builtin_htop_tagged_count_hint(count: usize) -> String {
    format!(
        "{count} {}",
        localized_message(MessageKey::BuiltinHtopTaggedCount, &[])
    )
}
