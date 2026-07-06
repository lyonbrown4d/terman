use std::{collections::BTreeMap, io, path::Path};

use portable_pty::CommandBuilder;

use crate::{
    ScreenArgs,
    shell::{default_shell, shell_command_args},
};

pub(crate) fn build_command(
    args: &ScreenArgs,
    cwd: Option<&Path>,
    env_overrides: &BTreeMap<String, Option<String>>,
) -> Result<CommandBuilder, io::Error> {
    let command_shell = default_shell();
    let (window_shell, window_login_shell) = default_window_shell(env_overrides);

    let mut builder = match args.command.clone() {
        Some(cmd) => {
            let mut builder = CommandBuilder::new(&command_shell);
            for arg in shell_command_args(&command_shell, args.login_shell) {
                builder.arg(arg);
            }
            builder.arg(cmd);
            builder
        }
        None => {
            if !cfg!(windows) && (args.login_shell || window_login_shell) {
                let mut builder = CommandBuilder::new(&window_shell);
                builder.arg("-l");
                builder
            } else {
                CommandBuilder::new(window_shell)
            }
        }
    };

    if let Some(path) = cwd {
        builder.cwd(path.as_os_str());
    }

    for (name, value) in env_overrides {
        match value {
            Some(value) => builder.env(name, value),
            None => builder.env_remove(name),
        }
    }
    apply_screen_environment(&mut builder, args);
    Ok(builder)
}

fn default_window_shell(env_overrides: &BTreeMap<String, Option<String>>) -> (String, bool) {
    let Some(Some(shell)) = env_overrides.get("SHELL") else {
        return (default_shell(), false);
    };
    let shell = shell.trim();
    if shell.is_empty() {
        return (default_shell(), false);
    }
    match shell.strip_prefix('-').filter(|value| !value.is_empty()) {
        Some(shell) => (shell.to_string(), true),
        None => (shell.to_string(), false),
    }
}

fn apply_screen_environment(builder: &mut CommandBuilder, args: &ScreenArgs) {
    builder.env("TERM", "screen-256color");
    builder.env("WINDOW", "0");

    if let Some(session_name) = &args.session_name {
        builder.env("STY", session_name.as_str());
        builder.env("TERMAN_SCREEN_SESSION", session_name.as_str());
    }
}