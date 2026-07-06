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
    let shell = default_shell();

    let mut builder = match args.command.clone() {
        Some(cmd) => {
            let mut builder = CommandBuilder::new(&shell);
            for arg in shell_command_args(&shell, args.login_shell) {
                builder.arg(arg);
            }
            builder.arg(cmd);
            builder
        }
        None => {
            if !cfg!(windows) && args.login_shell {
                let mut builder = CommandBuilder::new(&shell);
                builder.arg("-l");
                builder
            } else {
                CommandBuilder::new(shell)
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

fn apply_screen_environment(builder: &mut CommandBuilder, args: &ScreenArgs) {
    builder.env("TERM", "screen-256color");
    builder.env("WINDOW", "0");

    if let Some(session_name) = &args.session_name {
        builder.env("STY", session_name.as_str());
        builder.env("TERMAN_SCREEN_SESSION", session_name.as_str());
    }
}
