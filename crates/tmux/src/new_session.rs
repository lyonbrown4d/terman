use std::{error::Error, io, thread, time::Duration};

use crate::{
    args::session_name_arg,
    attach::attach_builtin_tmux_session,
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest},
    launcher::spawn_detached_tmux_server,
    service::request_endpoint_response,
    sessions::{builtin_tmux_session_exists, register_builtin_tmux_session},
};

pub(crate) fn create_builtin_tmux_session(
    args: &[String],
    attached: bool,
) -> Result<(), Box<dyn Error>> {
    let name = required_session_name_arg(args)?;
    if builtin_tmux_session_exists(&name)? {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::AlreadyExists,
            terman_common::builtin_tmux_session_exists_hint(&name),
        )));
    }

    let endpoint = TmuxIpcEndpoint::for_new_session(&name);
    let command_args = new_session_command_args(args);
    let server_pid = spawn_detached_tmux_server(&name, endpoint.raw_name(), &command_args)?;
    let command = session_command(&command_args);
    if !register_builtin_tmux_session(&name, Some(server_pid.to_string()), command, &endpoint)? {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::AlreadyExists,
            terman_common::builtin_tmux_session_exists_hint(&name),
        )));
    }

    println!("{}", terman_common::builtin_tmux_session_created_hint(&name));
    if attached {
        wait_for_session_server(&endpoint)?;
        attach_builtin_tmux_session(&attach_session_args(&name))?;
    }
    Ok(())
}

fn required_session_name_arg(args: &[String]) -> Result<String, Box<dyn Error>> {
    session_name_arg(args).ok_or_else(|| {
        Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_tmux_session_name_required_hint(),
        )) as Box<dyn Error>
    })
}

fn wait_for_session_server(endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
    let mut last_error = None;

    for _ in 0..50 {
        match request_endpoint_response(endpoint, TmuxIpcRequest::Ping) {
            Ok(_) => return Ok(()),
            Err(err) => {
                last_error = Some(err);
                thread::sleep(Duration::from_millis(20));
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        io::Error::new(io::ErrorKind::TimedOut, "tmux server did not become ready")
    }))
}

fn attach_session_args(session_name: &str) -> Vec<String> {
    vec![
        String::from("attach-session"),
        String::from("-t"),
        session_name.to_string(),
    ]
}

fn session_command(command_args: &[String]) -> Option<String> {
    if command_args.is_empty() {
        None
    } else {
        Some(command_args.join(" "))
    }
}

fn new_session_command_args(args: &[String]) -> Vec<String> {
    let mut seen_command = false;
    let mut command_started = false;
    let mut skip_next = false;
    let mut command_args = Vec::new();

    for arg in args {
        if command_started {
            command_args.push(arg.clone());
            continue;
        }
        if skip_next {
            skip_next = false;
            continue;
        }
        if !seen_command {
            if matches!(arg.as_str(), "new" | "new-session") {
                seen_command = true;
            }
            continue;
        }
        if matches!(arg.as_str(), "-d" | "--detached") {
            continue;
        }
        if matches!(arg.as_str(), "-s" | "--session-name") {
            skip_next = true;
            continue;
        }
        if arg.starts_with("-s") || arg.starts_with("--session-name=") || arg.starts_with('-') {
            continue;
        }
        command_started = true;
        command_args.push(arg.clone());
    }

    command_args
}

#[cfg(test)]
mod tests {
    use super::{attach_session_args, new_session_command_args, session_command};

    #[test]
    fn extracts_new_session_command_args() {
        let args = vec![
            String::from("new-session"),
            String::from("-s"),
            String::from("dev"),
            String::from("cargo"),
            String::from("watch"),
        ];

        assert_eq!(
            new_session_command_args(&args),
            vec![String::from("cargo"), String::from("watch")]
        );
    }

    #[test]
    fn builds_attach_args() {
        assert_eq!(
            attach_session_args("dev"),
            vec![String::from("attach-session"), String::from("-t"), String::from("dev")]
        );
    }

    #[test]
    fn joins_session_command() {
        assert_eq!(
            session_command(&[String::from("cargo"), String::from("watch")]),
            Some(String::from("cargo watch"))
        );
        assert_eq!(session_command(&[]), None);
    }
}