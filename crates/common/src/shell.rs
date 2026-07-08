use std::env;

pub fn shell_command_args(shell: &str, login_shell: bool) -> Vec<String> {
    let file_name = std::path::Path::new(shell)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(shell)
        .to_ascii_lowercase();

    if file_name.contains("cmd.exe") {
        return vec![String::from("/C")];
    }

    if file_name.contains("powershell") || file_name.contains("pwsh") {
        return vec![String::from("-Command")];
    }

    if file_name.ends_with("bash")
        || file_name.ends_with("bash.exe")
        || file_name.ends_with("sh")
        || file_name.ends_with("sh.exe")
    {
        if login_shell {
            return vec![String::from("-lc")];
        }
        return vec![String::from("-c")];
    }

    vec![String::from("-c")]
}

pub fn default_shell() -> String {
    if cfg!(windows) {
        env::var("COMSPEC")
            .ok()
            .filter(|value| !value.is_empty())
            .or_else(|| env::var("ComSpec").ok())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| String::from("cmd.exe"))
    } else {
        env::var("SHELL")
            .ok()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| String::from("/bin/sh"))
    }
}

#[cfg(test)]
mod tests {
    use super::shell_command_args;

    #[test]
    fn detects_shell_command_flags() {
        assert_eq!(shell_command_args("cmd.exe", false), vec![String::from("/C")]);
        assert_eq!(
            shell_command_args("pwsh.exe", false),
            vec![String::from("-Command")]
        );
        assert_eq!(shell_command_args("bash", true), vec![String::from("-lc")]);
        assert_eq!(shell_command_args("bash", false), vec![String::from("-c")]);
    }
}