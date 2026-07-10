pub(crate) fn session_name_arg(args: &[String]) -> Option<String> {
    named_arg(args, "-s", "--session-name")
}

pub(crate) fn target_session_arg(args: &[String]) -> Option<String> {
    named_arg(args, "-t", "--target-session")
}

pub(crate) fn target_session_name_arg(args: &[String]) -> Option<String> {
    target_session_arg(args).map(|target| match target.split_once(':') {
        Some((session, _)) => session.to_string(),
        None => target,
    })
}

pub(crate) fn target_window_selector_arg(args: &[String]) -> Option<String> {
    target_session_arg(args)
        .and_then(|target| target.split_once(':').map(|(_, selector)| selector.to_string()))
        .and_then(|selector| normalized_window_selector(&selector).map(ToString::to_string))
}

pub(crate) fn target_window_index_arg(args: &[String]) -> Option<usize> {
    target_window_selector_arg(args).and_then(|selector| selector.parse::<usize>().ok())
}

pub(crate) fn target_pane_index_arg(args: &[String]) -> Option<usize> {
    target_session_arg(args).and_then(|target| {
        target
            .split_once(':')
            .and_then(|(_, selector)| selector.rsplit_once('.'))
            .and_then(|(_, pane)| pane.parse::<usize>().ok())
    })
}

fn normalized_window_selector(selector: &str) -> Option<&str> {
    let selector = selector.trim();
    if selector.is_empty() { return None; }
    if let Some((window, pane)) = selector.rsplit_once('.') {
        if pane.parse::<usize>().is_ok() {
            return if window.is_empty() { Some(".") } else { Some(window) };
        }
    }
    Some(selector)
}
pub(crate) fn rename_session_name_arg(args: &[String]) -> Option<String> {
    positional_after_command(args, &["rename-session"])
}

pub(crate) fn rename_window_name_arg(args: &[String]) -> Option<String> {
    positional_after_command(args, &["rename-window", "renamew"])
}

pub(crate) fn new_window_name_arg(args: &[String]) -> Option<String> {
    named_arg(args, "-n", "--window-name")
}

pub(crate) fn new_window_command_arg(args: &[String]) -> Option<String> {
    let payload = new_window_command_args(args).join(" ");
    if payload.trim().is_empty() { None } else { Some(payload) }
}

pub(crate) fn kill_other_windows_arg(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "-a")
}
pub(crate) fn resize_pane_width_arg(args: &[String]) -> Option<u16> {
    named_arg(args, "-x", "--width").and_then(|value| value.parse::<u16>().ok())
}

pub(crate) fn resize_pane_height_arg(args: &[String]) -> Option<u16> {
    named_arg(args, "-y", "--height").and_then(|value| value.parse::<u16>().ok())
}
pub(crate) fn display_message_arg(args: &[String]) -> Option<String> {
    positional_payload_after_command(args, &["display-message", "display"])
}

pub(crate) fn send_keys_args(args: &[String]) -> Vec<String> {
    positional_args_after_command(args, &["send-keys", "send"])
}

fn new_window_command_args(args: &[String]) -> Vec<String> {
    let mut seen_command = false;
    let mut skip_next = false;
    let mut payload_started = false;
    let mut payload = Vec::new();

    for arg in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if !seen_command {
            if matches!(arg.as_str(), "new-window" | "neww") {
                seen_command = true;
            }
            continue;
        }
        if !payload_started && new_window_option_consumes_value(arg) {
            skip_next = true;
            continue;
        }
        if !payload_started && new_window_inline_option(arg) {
            continue;
        }
        if !payload_started && arg.starts_with('-') {
            continue;
        }
        payload_started = true;
        payload.push(arg.clone());
    }
    payload
}

fn new_window_option_consumes_value(arg: &str) -> bool {
    matches!(arg, "-t" | "--target-session" | "-n" | "--window-name")
}

fn new_window_inline_option(arg: &str) -> bool {
    arg.starts_with("-t")
        || arg.starts_with("--target-session=")
        || arg.starts_with("-n")
        || arg.starts_with("--window-name=")
}
fn positional_after_command(args: &[String], commands: &[&str]) -> Option<String> {
    positional_payload_after_command(args, commands).and_then(|payload| {
        payload
            .split_whitespace()
            .next()
            .map(ToString::to_string)
    })
}

fn positional_payload_after_command(args: &[String], commands: &[&str]) -> Option<String> {
    let payload = positional_args_after_command(args, commands).join(" ");
    if payload.trim().is_empty() {
        None
    } else {
        Some(payload)
    }
}

fn positional_args_after_command(args: &[String], commands: &[&str]) -> Vec<String> {
    let mut seen_command = false;
    let mut skip_next = false;
    let mut payload = Vec::new();

    for arg in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if !seen_command {
            if commands.contains(&arg.as_str()) {
                seen_command = true;
            }
            continue;
        }
        if arg == "-t" || arg == "--target-session" {
            skip_next = true;
            continue;
        }
        if arg.starts_with("-t") || arg.starts_with("--target-session=") || arg.starts_with('-') {
            continue;
        }
        payload.push(arg.clone());
    }

    payload
}

fn named_arg(args: &[String], short: &str, long: &str) -> Option<String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == short || arg == long {
            return iter.next().cloned();
        }
        if let Some(value) = arg.strip_prefix(short).filter(|value| !value.is_empty()) {
            return Some(value.to_string());
        }
        if let Some(value) = arg.strip_prefix(&format!("{long}=")) {
            return Some(value.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{
        display_message_arg, new_window_command_arg, new_window_name_arg, rename_session_name_arg, rename_window_name_arg, resize_pane_height_arg, resize_pane_width_arg, send_keys_args,
        session_name_arg, target_pane_index_arg, target_session_arg, target_session_name_arg, target_window_index_arg,
    };

    #[test]
    fn parses_session_name_arg() {
        assert_eq!(session_name_arg(&["new".into(), "-s".into(), "dev".into()]), Some(String::from("dev")));
        assert_eq!(session_name_arg(&["new".into(), "-sdev".into()]), Some(String::from("dev")));
        assert_eq!(session_name_arg(&["new".into(), "--session-name=dev".into()]), Some(String::from("dev")));
    }

    #[test]
    fn parses_target_session_arg() {
        assert_eq!(target_session_arg(&["kill-session".into(), "-t".into(), "dev".into()]), Some(String::from("dev")));
        assert_eq!(target_session_arg(&["kill-session".into(), "-tdev".into()]), Some(String::from("dev")));
        assert_eq!(target_session_arg(&["kill-session".into(), "--target-session=dev".into()]), Some(String::from("dev")));
    }

    #[test]
    fn parses_target_window_parts() {
        let args = ["rename-window".into(), "-t".into(), "dev:2".into()];
        assert_eq!(target_session_name_arg(&args), Some(String::from("dev")));
        assert_eq!(target_window_index_arg(&args), Some(2));
    }

    #[test]
    fn parses_target_pane_parts() {
        let args = ["kill-pane".into(), "-t".into(), "dev:2.0".into()];
        assert_eq!(target_session_name_arg(&args), Some(String::from("dev")));
        assert_eq!(target_window_index_arg(&args), Some(2));
        assert_eq!(target_pane_index_arg(&args), Some(0));
    }
    #[test]
    fn parses_rename_names() {
        assert_eq!(rename_session_name_arg(&["rename-session".into(), "-told".into(), "new".into()]), Some(String::from("new")));
        assert_eq!(rename_window_name_arg(&["renamew".into(), "--target-session=dev:0".into(), "api".into()]), Some(String::from("api")));
    }

    #[test]
    fn parses_display_message_payload() {
        assert_eq!(
            display_message_arg(&["display".into(), "-tdev".into(), "hello".into(), "world".into()]),
            Some(String::from("hello world"))
        );
    }

    #[test]
    fn parses_new_window_args() {
        let args = ["neww".into(), "-tdev".into(), "-n".into(), "api".into(), "cargo".into(), "run".into()];
        assert_eq!(new_window_name_arg(&args), Some(String::from("api")));
        assert_eq!(new_window_command_arg(&args), Some(String::from("cargo run")));
    }
    #[test]
    fn parses_resize_pane_size() {
        let args = ["resizep".into(), "-tdev:1.0".into(), "-x".into(), "120".into(), "-y40".into()];
        assert_eq!(resize_pane_width_arg(&args), Some(120));
        assert_eq!(resize_pane_height_arg(&args), Some(40));
    }
    #[test]
    fn parses_send_keys_payload() {
        assert_eq!(
            send_keys_args(&["send".into(), "-tdev".into(), "echo hi".into(), "Enter".into()]),
            vec![String::from("echo hi"), String::from("Enter")]
        );
    }
}
