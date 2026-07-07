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

pub(crate) fn target_window_index_arg(args: &[String]) -> Option<usize> {
    target_session_arg(args).and_then(|target| {
        target
            .split_once(':')
            .and_then(|(_, index)| index.parse::<usize>().ok())
    })
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
        display_message_arg, new_window_command_arg, new_window_name_arg, rename_session_name_arg, rename_window_name_arg, send_keys_args,
        session_name_arg, target_session_arg, target_session_name_arg, target_window_index_arg,
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
    fn parses_send_keys_payload() {
        assert_eq!(
            send_keys_args(&["send".into(), "-tdev".into(), "echo hi".into(), "Enter".into()]),
            vec![String::from("echo hi"), String::from("Enter")]
        );
    }
}
