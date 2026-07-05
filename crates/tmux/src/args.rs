pub(crate) fn session_name_arg(args: &[String]) -> Option<String> {
    named_arg(args, "-s", "--session-name")
}

pub(crate) fn target_session_arg(args: &[String]) -> Option<String> {
    named_arg(args, "-t", "--target-session")
}

pub(crate) fn rename_session_name_arg(args: &[String]) -> Option<String> {
    let mut seen_command = false;
    let mut skip_next = false;

    for arg in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if !seen_command {
            if arg == "rename-session" {
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
        return Some(arg.clone());
    }
    None
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
    use super::{rename_session_name_arg, session_name_arg, target_session_arg};

    #[test]
    fn parses_session_name_arg() {
        assert_eq!(
            session_name_arg(&["new".into(), "-s".into(), "dev".into()]),
            Some(String::from("dev"))
        );
        assert_eq!(
            session_name_arg(&["new".into(), "-sdev".into()]),
            Some(String::from("dev"))
        );
        assert_eq!(
            session_name_arg(&["new".into(), "--session-name=dev".into()]),
            Some(String::from("dev"))
        );
    }

    #[test]
    fn parses_target_session_arg() {
        assert_eq!(
            target_session_arg(&["kill-session".into(), "-t".into(), "dev".into()]),
            Some(String::from("dev"))
        );
        assert_eq!(
            target_session_arg(&["kill-session".into(), "-tdev".into()]),
            Some(String::from("dev"))
        );
        assert_eq!(
            target_session_arg(&["kill-session".into(), "--target-session=dev".into()]),
            Some(String::from("dev"))
        );
    }

    #[test]
    fn parses_rename_session_name_arg() {
        assert_eq!(
            rename_session_name_arg(&[
                "rename-session".into(),
                "-t".into(),
                "old".into(),
                "new".into(),
            ]),
            Some(String::from("new"))
        );
        assert_eq!(
            rename_session_name_arg(&[
                "rename-session".into(),
                "--target-session=old".into(),
                "new".into(),
            ]),
            Some(String::from("new"))
        );
    }
}
