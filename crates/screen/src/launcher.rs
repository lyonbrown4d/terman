use std::{
    env,
    error::Error,
    io,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use terman_common::{
    builtin_screen_named_session_required_hint, builtin_screen_server_timeout_hint,
};

use crate::{ScreenArgs, service::request_screen_attach};

const SERVER_ATTACH_ATTEMPTS: usize = 80;
const SERVER_ATTACH_RETRY_DELAY: Duration = Duration::from_millis(25);

pub(crate) fn run_named_screen_session(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    let session_name = args.session_name.clone().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            builtin_screen_named_session_required_hint(),
        )
    })?;

    spawn_screen_server(&args)?;

    let attach_args = ScreenArgs {
        resume: Some(Some(session_name)),
        ..ScreenArgs::default()
    };
    attach_when_ready(&attach_args)
}

fn spawn_screen_server(args: &ScreenArgs) -> io::Result<()> {
    let mut command = Command::new(env::current_exe()?);
    command
        .arg("--__screen-server")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    if let Some(command_text) = &args.command {
        command.arg("--command").arg(command_text);
    }
    if let Some(cols) = args.cols {
        command.arg("--cols").arg(cols.to_string());
    }
    if let Some(rows) = args.rows {
        command.arg("--rows").arg(rows.to_string());
    }
    if let Some(session_name) = &args.session_name {
        command.arg("-S").arg(session_name);
    }
    if args.login_shell {
        command.arg("--login-shell");
    }

    let _child = command.spawn()?;
    Ok(())
}

fn attach_when_ready(args: &ScreenArgs) -> Result<(), Box<dyn Error>> {
    let mut last_error: Option<io::Error> = None;

    for _ in 0..SERVER_ATTACH_ATTEMPTS {
        match request_screen_attach(args) {
            Ok(()) => return Ok(()),
            Err(err) => {
                last_error = Some(err);
                thread::sleep(SERVER_ATTACH_RETRY_DELAY);
            }
        }
    }

    Err(Box::new(last_error.unwrap_or_else(|| {
        io::Error::new(
            io::ErrorKind::TimedOut,
            builtin_screen_server_timeout_hint(),
        )
    })))
}
