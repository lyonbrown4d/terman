const BUFFER_COMMANDS: &[&str] = &["set-buffer", "setb"];

pub(crate) fn buffer_name_arg(args: &[String]) -> Option<String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "-b" || arg == "--buffer-name" {
            return iter.next().cloned().filter(|name| !name.is_empty());
        }
        if let Some(name) = arg.strip_prefix("-b").filter(|name| !name.is_empty()) {
            return Some(name.to_string());
        }
        if let Some(name) = arg.strip_prefix("--buffer-name=") {
            return (!name.is_empty()).then(|| name.to_string());
        }
    }
    None
}

pub(crate) fn buffer_data_arg(args: &[String]) -> Option<String> {
    let mut seen_command = false;
    let mut skip_next = false;
    let mut literal = false;
    let mut payload = Vec::new();

    for arg in args {
        if skip_next {
            skip_next = false;
            continue;
        }
        if !seen_command {
            seen_command = BUFFER_COMMANDS.contains(&arg.as_str());
            continue;
        }
        if literal {
            payload.push(arg.clone());
            continue;
        }
        if arg == "--" {
            literal = true;
            continue;
        }
        if option_consumes_value(arg) {
            skip_next = true;
            continue;
        }
        if inline_option(arg) || arg == "-a" || arg == "--append" {
            continue;
        }
        if arg.starts_with('-') {
            continue;
        }
        payload.push(arg.clone());
    }

    (!payload.is_empty()).then(|| payload.join(" "))
}

fn option_consumes_value(arg: &str) -> bool {
    matches!(
        arg,
        "-t" | "--target-session" | "-b" | "--buffer-name"
    )
}

fn inline_option(arg: &str) -> bool {
    arg.starts_with("-t")
        || arg.starts_with("--target-session=")
        || arg.starts_with("-b")
        || arg.starts_with("--buffer-name=")
}