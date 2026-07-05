use std::env;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use sys_locale::get_locale;
use unic_langid::LanguageIdentifier;

use super::MessageKey;

const ZH_CN_MESSAGES: &[u8] = include_bytes!("../../i18n/zh-CN.ftl");
const EN_US_MESSAGES: &[u8] = include_bytes!("../../i18n/en-US.ftl");

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
