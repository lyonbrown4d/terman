use std::{error::Error, io};

mod args;
mod builtin;
mod cli;
mod command;
mod ipc;
mod service;
mod session_core;
mod sessions;

pub use cli::{TmuxArgs, run_with_binary_parse};
use builtin::try_run_builtin_tmux_command;
use command::TmuxCommand;

pub fn run(args: TmuxArgs) -> Result<(), Box<dyn Error>> {
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



