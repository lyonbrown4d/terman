use std::io;

use portable_pty::CommandBuilder;

use crate::{
    ScreenArgs,
    shell::{default_shell, shell_command_args},
};

pub(crate) fn build_command(args: &ScreenArgs) -> Result<CommandBuilder, io::Error> {
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

    if let Some(session_name) = &args.session_name {
        builder.env("STY", session_name.as_str());
        builder.env("TERMAN_SCREEN_SESSION", session_name.as_str());
    }

    Ok(builder)
}