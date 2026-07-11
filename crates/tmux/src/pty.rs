#![allow(dead_code)]

use portable_pty::CommandBuilder;

use crate::shell::{default_shell, shell_command_args};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TmuxPtyCommandSpec {
    pub(crate) session_name: String,
    pub(crate) window_index: u32,
    pub(crate) window_name: String,
    pub(crate) pane_index: u32,
    pub(crate) command: Option<String>,
    pub(crate) login_shell: bool,
}

pub(crate) fn build_tmux_pty_command(spec: &TmuxPtyCommandSpec) -> CommandBuilder {
    let shell = default_shell();
    let mut builder = match spec.command.clone() {
        Some(command) => build_shell_command(&shell, spec.login_shell, command),
        None => build_interactive_shell(&shell, spec.login_shell),
    };
    apply_tmux_environment(&mut builder, spec);
    builder
}

fn build_shell_command(shell: &str, login_shell: bool, command: String) -> CommandBuilder {
    let mut builder = CommandBuilder::new(shell);
    for arg in shell_command_args(shell, login_shell) {
        builder.arg(arg);
    }
    builder.arg(command);
    builder
}

fn build_interactive_shell(shell: &str, login_shell: bool) -> CommandBuilder {
    if !cfg!(windows) && login_shell {
        let mut builder = CommandBuilder::new(shell);
        builder.arg("-l");
        builder
    } else {
        CommandBuilder::new(shell)
    }
}

fn apply_tmux_environment(builder: &mut CommandBuilder, spec: &TmuxPtyCommandSpec) {
    builder.env("TERM", "tmux-256color");
    builder.env("WINDOW", spec.window_index.to_string());
    builder.env("TMUX_PANE", format!("%{}", spec.pane_index));
    builder.env("TERMAN_TMUX_SESSION", spec.session_name.as_str());
    builder.env("TERMAN_TMUX_WINDOW", spec.window_name.as_str());
}

#[cfg(test)]
mod tests {
    use super::TmuxPtyCommandSpec;

    #[test]
    fn models_tmux_pty_command_spec() {
        let spec = TmuxPtyCommandSpec {
            session_name: String::from("dev"),
            window_index: 0,
            window_name: String::from("shell"),
            pane_index: 0,
            command: Some(String::from("echo hi")),
            login_shell: false,
        };
        assert_eq!(spec.session_name, "dev");
        assert_eq!(spec.window_name, "shell");
        assert_eq!(spec.pane_index, 0);
    }
}
