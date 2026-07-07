use std::error::Error;

mod builtin;
mod builtin_output;
mod builtin_mouse;
mod cli;
mod di;
mod ipc;
mod launcher;
mod pty;
mod pty_process;
mod service;
mod server;
mod session_core;
mod screen_exchange;
mod sessions;
mod shell;
mod terminal_input;
mod terminal_mouse;
mod window_runtime;

pub use cli::{ScreenArgs, run_with_binary_parse};
use builtin::run_builtin_screen;
use launcher::{
    run_detached_named_screen_session, run_named_screen_session,
    run_resume_or_create_screen_session,
};
use service::{request_screen_attach, request_screen_control_command};
use server::run_screen_server;
use sessions::{
    list_builtin_screen_sessions, list_builtin_screen_sessions_json, validate_screen_session_name,
    wipe_builtin_screen_sessions,
};

pub fn run(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    di::run(args)
}

pub(crate) fn run_command(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    if let Some(session_name) = &args.session_name {
        validate_screen_session_name(session_name)?;
    }
    if let Some(session_name) = &args.resume_or_create {
        validate_screen_session_name(session_name)?;
    }
    if let Some(Some(session_name)) = &args.resume {
        validate_screen_session_name(session_name)?;
    }
    if let Some(Some(session_name)) = &args.multi_attach {
        validate_screen_session_name(session_name)?;
    }

    if args.internal_server {
        return run_screen_server(args);
    }

    if args.resume_or_create.is_some() {
        return run_resume_or_create_screen_session(args);
    }

    if args.execute.is_some() {
        request_screen_control_command(&args)?;
        return Ok(());
    }

    if args.wipe {
        wipe_builtin_screen_sessions()?;
        return Ok(());
    }

    if args.detached {
        return run_detached_named_screen_session(args);
    }

    if is_builtin_screen_attach_requested(&args) {
        request_screen_attach(&args)?;
        return Ok(());
    }

    if args.list {
        if args.json {
            list_builtin_screen_sessions_json()?;
        } else {
            list_builtin_screen_sessions()?;
        }
        return Ok(());
    }

    if args.session_name.is_some() {
        return run_named_screen_session(args);
    }

    run_builtin_screen(args)
}

fn is_builtin_screen_attach_requested(args: &ScreenArgs) -> bool {
    args.resume.is_some() || args.multi_attach.is_some()
}

#[cfg(test)]
mod tests {
    use super::sessions::{
        BuiltinScreenSession, builtin_screen_session_is_alive, parse_builtin_screen_session_record,
        sanitize_session_file_name,
    };
    use super::{ScreenArgs, is_builtin_screen_attach_requested};
    use sysinfo::System;

    #[test]
    fn detects_builtin_attach_modes() {
        let resume = ScreenArgs {
            resume: Some(Some(String::from("dev"))),
            ..ScreenArgs::default()
        };
        let multi_attach = ScreenArgs {
            multi_attach: Some(None),
            ..ScreenArgs::default()
        };
        let new_session = ScreenArgs {
            session_name: Some(String::from("dev")),
            ..ScreenArgs::default()
        };

        assert!(is_builtin_screen_attach_requested(&resume));
        assert!(is_builtin_screen_attach_requested(&multi_attach));
        assert!(!is_builtin_screen_attach_requested(&new_session));
    }

    #[test]
    fn sanitizes_builtin_session_record_name() {
        assert_eq!(sanitize_session_file_name("dev/session:1"), "dev_session_1");
    }

    #[test]
    fn parses_builtin_session_record_without_legacy_ipc_endpoint() {
        let record = r#"{"name":"dev","pid":"42","cwd":"C:/repo","command":"pwsh"}"#;
        let parsed = parse_builtin_screen_session_record(record).expect("record should parse");

        assert_eq!(parsed.name, "dev");
        assert_eq!(parsed.pid, "42");
        assert_eq!(parsed.cwd, "C:/repo");
        assert_eq!(parsed.command, "pwsh");
        assert_eq!(parsed.ipc_endpoint, None);
    }

    #[test]
    fn treats_invalid_session_pid_as_dead() {
        let system = System::new();
        let session = BuiltinScreenSession {
            name: String::from("dev"),
            pid: String::from("not-a-pid"),
            cwd: String::from("C:/repo"),
            command: String::from("pwsh"),
            ipc_endpoint: None,
        };

        assert!(!builtin_screen_session_is_alive(&session, &system));
    }
}
