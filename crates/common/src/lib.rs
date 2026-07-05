use std::{
    env,
    ffi::OsString,
    io,
    process::{ExitStatus, Stdio},
    time::Duration,
};

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use sys_locale::get_locale;
use tokio::process::Command as TokioCommand;
use unic_langid::LanguageIdentifier;
use which::which;

pub const DEFAULT_COMMAND_TIMEOUT: Duration = Duration::from_secs(8);

const ZH_CN_MESSAGES: &[u8] = include_bytes!("../i18n/zh-CN.ftl");
const EN_US_MESSAGES: &[u8] = include_bytes!("../i18n/en-US.ftl");

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MessageKey {
    NativeToolNotFound,
    BuiltinScreenNoSessions,
    BuiltinScreenSessionListHeader,
    BuiltinScreenSessionExists,
    BuiltinScreenAttachUnsupported,
    BuiltinScreenAttachTargetRequired,
    BuiltinScreenSessionNotFound,
    BuiltinScreenNamedSessionRequired,
    BuiltinScreenServerTimeout,
    BuiltinScreenControlCommandRequired,
    BuiltinScreenControlCommandUnsupported,
}

impl MessageKey {
    fn fluent_id(self) -> &'static str {
        match self {
            Self::NativeToolNotFound => "native-tool-not-found",
            Self::BuiltinScreenNoSessions => "builtin-screen-no-sessions",
            Self::BuiltinScreenSessionListHeader => "builtin-screen-session-list-header",
            Self::BuiltinScreenSessionExists => "builtin-screen-session-exists",
            Self::BuiltinScreenAttachUnsupported => "builtin-screen-attach-unsupported",
            Self::BuiltinScreenAttachTargetRequired => "builtin-screen-attach-target-required",
            Self::BuiltinScreenSessionNotFound => "builtin-screen-session-not-found",
            Self::BuiltinScreenNamedSessionRequired => "builtin-screen-named-session-required",
            Self::BuiltinScreenServerTimeout => "builtin-screen-server-timeout",
            Self::BuiltinScreenControlCommandRequired => "builtin-screen-control-command-required",
            Self::BuiltinScreenControlCommandUnsupported => "builtin-screen-control-command-unsupported",
        }
    }
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

pub fn builtin_screen_no_sessions_hint() -> String {
    localized_message(MessageKey::BuiltinScreenNoSessions, &[])
}

pub fn builtin_screen_session_list_header() -> String {
    localized_message(MessageKey::BuiltinScreenSessionListHeader, &[])
}

pub fn builtin_screen_session_exists_hint(name: &str) -> String {
    localized_message(MessageKey::BuiltinScreenSessionExists, &[("name", name)])
}

pub fn builtin_screen_attach_unsupported_hint() -> String {
    localized_message(MessageKey::BuiltinScreenAttachUnsupported, &[])
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

pub fn command_status_with_timeout(
    command: &str,
    args: &[&str],
    timeout: Duration,
) -> io::Result<Option<ExitStatus>> {
    let command = command.to_string();
    let args: Vec<OsString> = args.iter().map(OsString::from).collect();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()?;

    runtime.block_on(command_status_with_timeout_async(command, args, timeout))
}

pub async fn command_status_with_timeout_async(
    command: String,
    args: Vec<OsString>,
    timeout: Duration,
) -> io::Result<Option<ExitStatus>> {
    let mut child = TokioCommand::new(command)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()?;

    match tokio::time::timeout(timeout, child.wait()).await {
        Ok(status) => status.map(Some),
        Err(_) => Ok(None),
    }
}

pub fn which_binary(name: &str) -> Option<String> {
    which(name)
        .ok()
        .map(|path| path.to_string_lossy().to_string())
}

pub fn passthrough_env() -> Vec<(String, String)> {
    [
        "TERM",
        "COLORTERM",
        "LC_ALL",
        "LANG",
        "LC_CTYPE",
        "TERM_PROGRAM",
        "TERM_PROGRAM_VERSION",
    ]
    .iter()
    .filter_map(|k| env::var(k).ok().map(|v| (k.to_string(), v)))
    .collect()
}

pub fn terminal_env() -> Vec<(String, String)> {
    let mut vars = passthrough_env();
    if !vars.iter().any(|(key, _)| key == "TERM") {
        vars.push((String::from("TERM"), String::from("xterm-256color")));
    }
    vars
}

#[cfg(test)]
mod tests {
    use super::{
        MessageKey, MessageLanguage, builtin_screen_attach_target_required_hint,
        builtin_screen_attach_unsupported_hint, builtin_screen_no_sessions_hint,
        builtin_screen_session_exists_hint, builtin_screen_session_list_header,
        builtin_screen_session_not_found_hint, command_status_with_timeout,
        localized_message_for_language, message_language_from_tag, native_tool_not_found_hint,
    };
    use std::time::Duration;

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

    #[test]
    fn renders_chinese_native_tool_message_from_resource() {
        let message = localized_message_for_language(
            MessageLanguage::ZhCn,
            MessageKey::NativeToolNotFound,
            &[("tool", "screen")],
        );

        assert!(message.contains("screen"));
        assert!(message.contains("本机 screen 可执行文件"));
    }

    #[test]
    fn native_tool_not_found_hint_mentions_tool() {
        let hint = native_tool_not_found_hint("screen");
        assert!(hint.contains("screen"));
    }

    #[test]
    fn renders_builtin_screen_session_messages_from_resources() {
        assert!(builtin_screen_no_sessions_hint().contains("screen"));
        assert!(builtin_screen_session_list_header().contains("screen"));
        assert!(builtin_screen_session_exists_hint("dev").contains("dev"));
        assert!(builtin_screen_attach_unsupported_hint().contains("screen"));
        assert!(builtin_screen_attach_target_required_hint().contains("screen"));
        assert!(builtin_screen_session_not_found_hint("dev").contains("dev"));
    }

    #[test]
    fn command_status_with_timeout_returns_status_for_successful_command() {
        let status = if cfg!(windows) {
            command_status_with_timeout("cmd", &["/C", "exit 0"], Duration::from_secs(2))
        } else {
            command_status_with_timeout("sh", &["-c", "exit 0"], Duration::from_secs(2))
        }
        .expect("command should spawn");

        assert!(
            status
                .expect("command should finish before timeout")
                .success()
        );
    }

    #[test]
    fn command_status_with_timeout_returns_error_for_missing_command() {
        let result = command_status_with_timeout(
            "terman-definitely-missing-command",
            &[],
            Duration::from_secs(2),
        );

        assert!(result.is_err());
    }
}
