use std::{
    error::Error,
    io,
    process::{Command, ExitStatus, Stdio},
};

mod builtin;
mod cli;
mod command;
mod hints;
mod sessions;

pub use cli::{TmuxArgs, run_with_binary_parse};
use builtin::try_run_builtin_tmux_command;
use command::TmuxCommand;
use hints::{
    is_tmux_detached_without_tmux_command, tmux_failure_message, tmux_has_detached_arg,
    tmux_launch_failure_hint, tmux_runtime_hints,
};

struct TmuxLaunch {
    cmd: String,
}

pub fn run(args: TmuxArgs) -> Result<(), Box<dyn Error>> {
    let mut passed_args = args.args;
    let tmux_command = TmuxCommand::parse(&passed_args);
    if try_run_builtin_tmux_command(&tmux_command, &passed_args, args.detached)? {
        return Ok(());
    }

    let launch = resolve_tmux_launch()?;
    validate_tmux_launch(&launch)?;

    let mut cmd = Command::new(&launch.cmd);
    if args.detached {
        if tmux_command.is_new_session() {
            if !tmux_has_detached_arg(&passed_args) {
                passed_args.insert(0, String::from("-d"));
            }
        } else {
            eprintln!(
                "提示：--detached 通常与 `new/new-session` 配合使用；当前命令将按透传参数原样执行。"
            );
            if is_tmux_detached_without_tmux_command(&passed_args) {
                eprintln!(
                    "提示：当前只传了 -d/--detached 未带子命令时易触发预期外行为。建议显式指定 new/new-session 再启动。\n示例：`terman-tmux --detached new -s <name>`"
                );
            }
        }
    }

    let status: ExitStatus = cmd
        .args(&passed_args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .envs(terman_common::terminal_env())
        .status()?;

    let exit_code = status.code().unwrap_or(-1);
    if status.success() {
        Ok(())
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            tmux_failure_message(
                "tmux",
                exit_code,
                &format!(
                    "{}\n{}",
                    tmux_launch_failure_hint(),
                    tmux_runtime_hints(&passed_args, exit_code),
                ),
            ),
        )))
    }
}

fn validate_tmux_launch(launch: &TmuxLaunch) -> Result<(), Box<dyn Error>> {
    let status: ExitStatus = Command::new(&launch.cmd)
        .arg("-V")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;

    if status.success() {
        Ok(())
    } else {
        let code = status.code().unwrap_or(-1);
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            tmux_failure_message("tmux 可用性检查", code, &tmux_launch_failure_hint()),
        )))
    }
}

fn resolve_tmux_launch() -> Result<TmuxLaunch, Box<dyn Error>> {
    if let Some(path) = terman_common::which_binary("tmux") {
        return Ok(TmuxLaunch { cmd: path });
    }

    Err(Box::new(io::Error::new(
        io::ErrorKind::NotFound,
        tmux_launch_failure_hint(),
    )))
}