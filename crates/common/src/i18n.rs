use std::env;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use sys_locale::get_locale;
use unic_langid::LanguageIdentifier;

const ZH_CN_MESSAGES: &[u8] = include_bytes!("../i18n/zh-CN.ftl");
const EN_US_MESSAGES: &[u8] = include_bytes!("../i18n/en-US.ftl");

mod key;
pub use key::MessageKey;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum MessageLanguage {
    ZhCn,
    EnUs,
}

pub fn localized_message(key: MessageKey, vars: &[(&str, &str)]) -> String {
    localized_message_for_language(current_message_language(), key, vars)
}

pub fn native_tool_not_found_hint(tool: &str) -> String {
    localized_message(MessageKey::NativeToolNotFound, &[("tool", tool)])
}

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
pub fn builtin_screen_no_sessions_hint() -> String {
    localized_message(MessageKey::BuiltinScreenNoSessions, &[])
}

pub fn builtin_screen_session_list_header() -> String {
    localized_message(MessageKey::BuiltinScreenSessionListHeader, &[])
}

pub fn builtin_screen_session_exists_hint(name: &str) -> String {
    localized_message(MessageKey::BuiltinScreenSessionExists, &[("name", name)])
}

pub fn builtin_screen_session_name_empty_hint() -> String {
    localized_message(MessageKey::BuiltinScreenSessionNameEmpty, &[])
}

pub fn builtin_screen_attach_unsupported_hint() -> String {
    localized_message(MessageKey::BuiltinScreenAttachUnsupported, &[])
}

pub fn builtin_screen_attach_help_hint() -> String {
    localized_message(MessageKey::BuiltinScreenAttachHelp, &[])
}

pub fn builtin_screen_attach_target_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenAttachTargetRequired, &[])
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

pub fn builtin_screen_control_info_hint(
    session_name: &str,
    replay_bytes: usize,
    attach_clients: usize,
    cols: Option<u16>,
    rows: Option<u16>,
) -> String {
    let replay_bytes = replay_bytes.to_string();
    let attach_clients = attach_clients.to_string();
    let cols = cols.map(|value| value.to_string()).unwrap_or_else(|| String::from("?"));
    let rows = rows.map(|value| value.to_string()).unwrap_or_else(|| String::from("?"));
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

pub fn builtin_screen_control_hardcopy_path_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlHardcopyPathRequired, &[])
}

pub fn builtin_screen_control_pastefile_path_required_hint() -> String {
    localized_message(MessageKey::BuiltinScreenControlPastefilePathRequired, &[])
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

fn localized_message_for_language(
    language: MessageLanguage,
    key: MessageKey,
    vars: &[(&str, &str)],
) -> String {
    let Ok(messages) = std::str::from_utf8(messages_for_language(language)) else {
        return fallback_message(key, vars);
    };
    let Ok(resource) = FluentResource::try_new(messages.to_owned()) else {
        return fallback_message(key, vars);
    };

    let mut bundle = FluentBundle::new(vec![language_identifier(language)]);
    if bundle.add_resource(resource).is_err() {
        return fallback_message(key, vars);
    }

    let Some(message) = bundle.get_message(key.fluent_id()) else {
        return fallback_message(key, vars);
    };
    let Some(pattern) = message.value() else {
        return fallback_message(key, vars);
    };

    let mut args = FluentArgs::new();
    for (name, value) in vars {
        args.set(*name, *value);
    }

    let mut errors = Vec::new();
    bundle
        .format_pattern(pattern, Some(&args), &mut errors)
        .into_owned()
}

fn current_message_language() -> MessageLanguage {
    env::var("TERMAN_LANG")
        .ok()
        .or_else(get_locale)
        .map(|locale| message_language_from_tag(&locale))
        .unwrap_or(MessageLanguage::EnUs)
}

fn message_language_from_tag(tag: &str) -> MessageLanguage {
    let normalized = tag.replace('_', "-").to_ascii_lowercase();
    if normalized.starts_with("zh") {
        MessageLanguage::ZhCn
    } else {
        MessageLanguage::EnUs
    }
}

fn messages_for_language(language: MessageLanguage) -> &'static [u8] {
    match language {
        MessageLanguage::ZhCn => ZH_CN_MESSAGES,
        MessageLanguage::EnUs => EN_US_MESSAGES,
    }
}

fn language_identifier(language: MessageLanguage) -> LanguageIdentifier {
    match language {
        MessageLanguage::ZhCn => "zh-CN",
        MessageLanguage::EnUs => "en-US",
    }
    .parse()
    .expect("static language identifier should parse")
}

fn fallback_message(key: MessageKey, vars: &[(&str, &str)]) -> String {
    let mut message = key.fluent_id().to_string();
    for (name, value) in vars {
        message.push(' ');
        message.push_str(name);
        message.push('=');
        message.push_str(value);
    }
    message
}

#[cfg(test)]
mod tests {
    use super::{
        MessageKey, MessageLanguage, localized_message_for_language, message_language_from_tag,
    };

    #[test]
    fn detects_message_language_from_locale_tag() {
        assert_eq!(message_language_from_tag("zh-CN"), MessageLanguage::ZhCn);
        assert_eq!(message_language_from_tag("zh_TW"), MessageLanguage::ZhCn);
        assert_eq!(message_language_from_tag("en-US"), MessageLanguage::EnUs);
    }

    #[test]
    fn renders_english_native_tool_message_from_resource() {
        let message = localized_message_for_language(
            MessageLanguage::EnUs,
            MessageKey::NativeToolNotFound,
            &[("tool", "tmux")],
        );

        assert!(message.contains("tmux"));
        assert!(message.contains("native tmux executable"));
    }
}



