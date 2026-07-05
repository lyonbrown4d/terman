use std::{cell::RefCell, env};

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use sys_locale::get_locale;
use unic_langid::LanguageIdentifier;

use super::MessageKey;

const ZH_CN_MESSAGES: &[u8] = include_bytes!("../../i18n/zh-CN.ftl");
const EN_US_MESSAGES: &[u8] = include_bytes!("../../i18n/en-US.ftl");

thread_local! {
    static ZH_CN_BUNDLE: RefCell<Option<FluentBundle<FluentResource>>> = const { RefCell::new(None) };
    static EN_US_BUNDLE: RefCell<Option<FluentBundle<FluentResource>>> = const { RefCell::new(None) };
}

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
    format_with_cached_bundle(language, key, vars).unwrap_or_else(|| fallback_message(key, vars))
}

fn format_with_cached_bundle(
    language: MessageLanguage,
    key: MessageKey,
    vars: &[(&str, &str)],
) -> Option<String> {
    match language {
        MessageLanguage::ZhCn => ZH_CN_BUNDLE.with(|bundle| {
            let mut bundle = bundle.borrow_mut();
            format_with_bundle_slot(&mut bundle, language, key, vars)
        }),
        MessageLanguage::EnUs => EN_US_BUNDLE.with(|bundle| {
            let mut bundle = bundle.borrow_mut();
            format_with_bundle_slot(&mut bundle, language, key, vars)
        }),
    }
}

fn format_with_bundle_slot(
    bundle: &mut Option<FluentBundle<FluentResource>>,
    language: MessageLanguage,
    key: MessageKey,
    vars: &[(&str, &str)],
) -> Option<String> {
    if bundle.is_none() {
        *bundle = build_bundle(language);
    }
    format_with_bundle(bundle.as_ref()?, key, vars)
}

fn format_with_bundle(
    bundle: &FluentBundle<FluentResource>,
    key: MessageKey,
    vars: &[(&str, &str)],
) -> Option<String> {
    let message = bundle.get_message(key.fluent_id())?;
    let pattern = message.value()?;

    let mut args = FluentArgs::new();
    for (name, value) in vars {
        args.set(*name, *value);
    }

    let mut errors = Vec::new();
    Some(
        bundle
            .format_pattern(pattern, Some(&args), &mut errors)
            .into_owned(),
    )
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

fn build_bundle(language: MessageLanguage) -> Option<FluentBundle<FluentResource>> {
    let messages = std::str::from_utf8(messages_for_language(language)).ok()?;
    let resource = FluentResource::try_new(messages.to_owned()).ok()?;
    let mut bundle = FluentBundle::new(vec![language_identifier(language)]);
    bundle.add_resource(resource).ok()?;
    Some(bundle)
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