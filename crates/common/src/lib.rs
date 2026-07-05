use std::{
    env,
    ffi::OsString,
    io,
    process::{ExitStatus, Stdio},
    time::Duration,
};

use tokio::process::Command as TokioCommand;
use which::which;

pub const DEFAULT_COMMAND_TIMEOUT: Duration = Duration::from_secs(8);

const ZH_CN_MESSAGES: &str = r#"
wsl-install-hint = 建议先在 WSL 内执行 `wsl -e which {$tool}` / `wsl -e {$tool} -V` 确认安装与版本。
wsl-precheck-not-found-hint = 当前已进入 WSL 回退路径，但未检测到 WSL 内 {$tool}。请先安装：wsl -e sudo apt install {$tool}。
wsl-runtime-hint = 建议先执行 `wsl -l -v`（检查发行版）、`wsl --status`（检查子系统）与 `wsl -e {$tool} -V`（确认 {$tool} 可用）。
"#;

const EN_US_MESSAGES: &str = r#"
wsl-install-hint = Run `wsl -e which {$tool}` / `wsl -e {$tool} -V` inside WSL to confirm installation and version.
wsl-precheck-not-found-hint = The WSL fallback path is active, but {$tool} was not found inside WSL. Install it first with: wsl -e sudo apt install {$tool}.
wsl-runtime-hint = Run `wsl -l -v` to inspect distributions, `wsl --status` to inspect WSL, and `wsl -e {$tool} -V` to confirm {$tool} is available.
"#;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MessageKey {
    WslInstallHint,
    WslPrecheckNotFoundHint,
    WslRuntimeHint,
}

impl MessageKey {
    fn fluent_id(self) -> &'static str {
        match self {
            Self::WslInstallHint => "wsl-install-hint",
            Self::WslPrecheckNotFoundHint => "wsl-precheck-not-found-hint",
            Self::WslRuntimeHint => "wsl-runtime-hint",
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

fn localized_message_for_language(
    language: MessageLanguage,
    key: MessageKey,
    vars: &[(&str, &str)],
) -> String {
    let Ok(resource) = FluentResource::try_new(messages_for_language(language).to_string()) else {
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

fn messages_for_language(language: MessageLanguage) -> &'static str {
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
    let tool = vars
        .iter()
        .find_map(|(name, value)| (*name == "tool").then_some(*value))
        .unwrap_or("tool");

    match key {
        MessageKey::WslInstallHint => format!(
            "Run `wsl -e which {tool}` / `wsl -e {tool} -V` inside WSL to confirm installation and version."
        ),
        MessageKey::WslPrecheckNotFoundHint => format!(
            "The WSL fallback path is active, but {tool} was not found inside WSL. Install it first with: wsl -e sudo apt install {tool}."
        ),
        MessageKey::WslRuntimeHint => format!(
            "Run `wsl -l -v`, `wsl --status`, and `wsl -e {tool} -V` to confirm WSL and {tool} are available."
        ),
    }
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

pub fn wsl_which_status_with_timeout(
    wsl_command: &str,
    tool: &str,
    timeout: Duration,
) -> io::Result<Option<ExitStatus>> {
    command_status_with_timeout(wsl_command, &["-e", "which", tool], timeout)
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

pub fn which_wsl_binary() -> Option<String> {
    which_binary("wsl").or_else(|| which_binary("wsl.exe"))
}

pub fn wsl_install_hint(tool: &str) -> String {
    format!("建议先在 WSL 内执行 `wsl -e which {tool}` / `wsl -e {tool} -V` 确认安装与版本。")
}

pub fn wsl_precheck_not_found_hint(tool: &str) -> String {
    format!(
        "当前已进入 WSL 回退路径，但未检测到 WSL 内 {tool}。请先安装：wsl -e sudo apt install {tool}。"
    )
}

pub fn wsl_runtime_hint(tool: &str) -> String {
    format!(
        "建议先执行 `wsl -l -v`（检查发行版）、`wsl --status`（检查子系统）与 `wsl -e {tool} -V`（确认 {tool} 可用）。"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        MessageKey, MessageLanguage, command_status_with_timeout, localized_message_for_language,
        message_language_from_tag, wsl_precheck_not_found_hint, wsl_runtime_hint,
    };
    use std::time::Duration;

    #[test]
    fn wsl_precheck_not_found_hint_mentions_tool_and_install_cmd() {
        let tool = "tmux";
        let hint = wsl_precheck_not_found_hint(tool);

        assert!(hint.contains("未检测到 WSL 内 tmux"));
        assert!(hint.contains("wsl -e sudo apt install tmux"));
    }

    #[test]
    fn wsl_runtime_hint_uses_wsl_version_check_for_tool() {
        let tool = "screen";
        let hint = wsl_runtime_hint(tool);

        assert!(hint.contains("wsl -l -v"));
        assert!(hint.contains("wsl --status"));
        assert!(hint.contains("wsl -e screen -V"));
    }

    #[test]
    fn detects_message_language_from_locale_tag() {
        assert_eq!(message_language_from_tag("zh-CN"), MessageLanguage::ZhCn);
        assert_eq!(message_language_from_tag("zh_TW"), MessageLanguage::ZhCn);
        assert_eq!(message_language_from_tag("en-US"), MessageLanguage::EnUs);
    }

    #[test]
    fn renders_english_wsl_message_with_fluent() {
        let message = localized_message_for_language(
            MessageLanguage::EnUs,
            MessageKey::WslRuntimeHint,
            &[("tool", "tmux")],
        );

        assert!(message.contains("wsl -l -v"));
        assert!(message.contains("wsl --status"));
        assert!(message.contains("wsl -e tmux -V"));
    }

    #[test]
    fn renders_chinese_wsl_message_with_fluent() {
        let message = localized_message_for_language(
            MessageLanguage::ZhCn,
            MessageKey::WslInstallHint,
            &[("tool", "screen")],
        );

        assert!(message.contains("wsl -e which screen"));
        assert!(message.contains("wsl -e screen -V"));
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
