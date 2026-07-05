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
    builtin_screen_session_exists_hint,
};

use crate::{
    ScreenArgs,
    ipc::ScreenIpcEndpoint,
    service::{request_screen_attach, request_screen_server_ready},
    sessions::find_builtin_screen_session_for_attach,
};

const SERVER_ATTACH_ATTEMPTS: usize = 80;
const SERVER_ATTACH_RETRY_DELAY: Duration = Duration::from_millis(25);

pub(crate) fn run_detached_named_screen_session(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    let session_name = require_session_name(&args)?;
    ensure_named_session_available(&session_name)?;
    let endpoint = spawn_screen_server(&args)?;
    wait_until_ready(&endpoint)
}

pub(crate) fn run_resume_or_create_screen_session(
    mut args: ScreenArgs,
) -> Result<(), Box<dyn Error>> {
    let session_name = args.resume_or_create.clone().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            builtin_screen_named_session_required_hint(),
        )
    })?;

    let attach_args = ScreenArgs {
        resume: Some(Some(session_name.clone())),
        detach_existing: args.detach_existing,
        ..ScreenArgs::default()
    };
    match request_screen_attach(&attach_args) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            args.resume_or_create = None;
            args.session_name = Some(session_name);
            run_named_screen_session(args)
        }
        Err(err) => Err(Box::new(err)),
    }
}

pub(crate) fn run_named_screen_session(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    let session_name = require_session_name(&args)?;
    ensure_named_session_available(&session_name)?;

    spawn_screen_server(&args)?;

    let attach_args = ScreenArgs {
        resume: Some(Some(session_name)),
        ..ScreenArgs::default()
    };
    attach_when_ready(&attach_args)
}

fn require_session_name(args: &ScreenArgs) -> io::Result<String> {
    args.session_name.clone().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            builtin_screen_named_session_required_hint(),
        )
    })
}

fn ensure_named_session_available(session_name: &str) -> io::Result<()> {
    match find_builtin_screen_session_for_attach(Some(session_name)) {
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            builtin_screen_session_exists_hint(session_name),
        )),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

fn spawn_screen_server(args: &ScreenArgs) -> io::Result<ScreenIpcEndpoint> {
    let session_name = require_session_name(args)?;
    let endpoint = ScreenIpcEndpoint::for_new_session(&session_name);
    let mut command = Command::new(env::current_exe()?);
    command
        .arg("--__screen-server")
        .arg("--__endpoint-name")
        .arg(endpoint.raw_name())
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
    command.arg("-S").arg(session_name);
    if args.login_shell {
        command.arg("--login-shell");
    }

    let _child = command.spawn()?;
    Ok(endpoint)
}

fn wait_until_ready(endpoint: &ScreenIpcEndpoint) -> Result<(), Box<dyn Error>> {
    let mut last_error: Option<io::Error> = None;

    for _ in 0..SERVER_ATTACH_ATTEMPTS {
        match request_screen_server_ready(endpoint) {
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

