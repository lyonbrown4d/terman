use std::{error::Error, io};

mod args;
mod attach;
mod attach_command_exec;
mod attach_command_prompt;
mod copy_mode;
mod attach_copy;
mod attach_input;
mod attach_keys;
mod attach_mouse;
mod attach_pane;
mod attach_rename;
mod attach_status;
mod attach_window;
mod attach_window_list;
mod builtin;
mod buffer_commands;
mod buffer_args;
mod capture;
mod cli;
mod clients;
mod command;
mod detach_client;
mod di;
mod history;
mod ipc;
mod lifecycle;
mod new_session;
mod pane_commands;
mod pane_resize_args;
mod pane_select_command;
mod pane_layout;
mod pane_layout_swap;
mod pane_runtime;
mod pane_service;
mod pane_swap_command;
mod launcher;
mod refresh_client;
mod message;
mod pty;
mod service;
mod service_buffer;
mod service_codec;
mod service_handlers;
mod send_keys;
mod server;
mod session_core;
mod session_buffer;
mod session_model;
mod session_state;
mod sessions;
mod status;
mod shell;
mod terminal_frame;
mod terminal_mouse;
mod window_command_support;
mod window_commands;
mod window_option_service;
mod window_options;
mod window_runtime;
mod window_view;

pub use cli::{TmuxArgs, run_with_binary_parse};
use builtin::try_run_builtin_tmux_command;
use command::TmuxCommand;
use server::{TmuxServerConfig, run_tmux_server};

pub fn run(args: TmuxArgs) -> Result<(), Box<dyn Error>> {
    di::run(args)
}

pub(crate) fn run_command(args: TmuxArgs) -> Result<(), Box<dyn Error>> {
    if args.internal_server {
        return run_tmux_server(TmuxServerConfig::from_args(args)?);
    }

    let passed_args = args.args;
    let tmux_command = TmuxCommand::parse(&passed_args);
    if try_run_builtin_tmux_command(&tmux_command, &passed_args, args.detached)? {
        return Ok(());
    }

    Err(Box::new(io::Error::new(
        io::ErrorKind::Unsupported,
        terman_common::builtin_tmux_command_unsupported_hint(&unsupported_command_name(
            &passed_args,
        )),
    )))
}

fn unsupported_command_name(args: &[String]) -> String {
    args.iter()
        .find(|arg| arg.as_str() != "-d" && arg.as_str() != "--detached")
        .cloned()
        .unwrap_or_else(|| String::from("unknown"))
}

#[cfg(test)]
mod tests {
    use super::unsupported_command_name;

    #[test]
    fn detects_unsupported_command_name() {
        assert_eq!(
            unsupported_command_name(&["-d".into(), "attach".into()]),
            "attach"
        );
        assert_eq!(unsupported_command_name(&["--detached".into()]), "unknown");
    }
}
