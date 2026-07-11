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

pub fn builtin_screen_control_chdir_directory_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlChdirDirectoryRequired, &[])
}

pub fn builtin_screen_control_chdir_home_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlChdirHomeRequired, &[])
}

pub fn builtin_screen_control_echo_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlEchoRequired, &[])
}

pub fn builtin_screen_control_lastmsg_empty_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlLastmsgEmpty, &[])
}

pub fn builtin_screen_control_setenv_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlSetenvRequired, &[])
}

pub fn builtin_screen_control_unsetenv_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlUnsetenvRequired, &[])
}

pub fn builtin_screen_control_env_name_invalid_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlEnvNameInvalid, &[])
}

pub fn builtin_screen_control_shell_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlShellRequired, &[])
}

pub fn builtin_screen_control_shelltitle_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlShelltitleRequired, &[])
}

pub fn builtin_screen_control_term_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlTermRequired, &[])
}

pub fn builtin_screen_control_log_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlLogRequired, &[])
}

pub fn builtin_screen_control_monitor_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlMonitorRequired, &[])
}

pub fn builtin_screen_control_silence_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlSilenceRequired, &[])
}

pub fn builtin_screen_control_logfile_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlLogfileRequired, &[])
}

pub fn builtin_screen_control_logtstamp_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlLogtstampRequired, &[])
}

pub fn builtin_screen_control_stuff_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlStuffRequired, &[])
}

pub fn builtin_screen_control_register_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlRegisterRequired, &[])
}

pub fn builtin_screen_control_resize_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlResizeRequired, &[])
}

pub fn builtin_screen_control_size_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlSizeRequired, &[])
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

pub fn builtin_screen_control_number_hint(index: usize, title: &str) -> String {
    let index = index.to_string();
    localized_message(
        MessageKey::BuiltinScreenControlNumber,
        &[("index", &index), ("title", title)],
    )
}

pub fn builtin_screen_control_number_invalid_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlNumberInvalid, &[])
}

pub fn builtin_screen_control_scrollback_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlScrollbackRequired, &[])
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

pub fn builtin_screen_control_title_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlTitleRequired, &[])
}

pub fn builtin_screen_control_version_hint(version: &str) -> String {
    localized_message(MessageKey::BuiltinScreenControlVersion, &[("version", version)])
}

pub fn builtin_screen_control_license_hint(version: &str) -> String {
    localized_message(MessageKey::BuiltinScreenControlLicense, &[("version", version)])
}