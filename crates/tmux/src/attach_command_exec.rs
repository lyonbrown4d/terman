use std::io;

use crate::{
    builtin::try_run_builtin_tmux_command,
    command::TmuxCommand,
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
};

pub(crate) fn execute_attached_command(
    endpoint: &TmuxIpcEndpoint,
    client_id: &str,
    line: &str,
) -> io::Result<bool> {
    let mut args = split_command_line(line).map_err(|()| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_tmux_command_parse_error_hint(),
        )
    })?;
    if args.is_empty() {
        return Ok(false);
    }
    let command = TmuxCommand::parse(&args);
    if command == TmuxCommand::AttachSession {
        return Err(unsupported_command(args.first()));
    }
    if command == TmuxCommand::DetachClient && args.len() == 1 {
        send_detach(endpoint, client_id)?;
        return Ok(true);
    }
    let session = current_session_name(endpoint)?;
    bind_attached_session(&command, &mut args, session.as_str());
    let handled = try_run_builtin_tmux_command(&command, &args, true)
        .map_err(|error| io::Error::other(error.to_string()))?;
    if !handled {
        return Err(unsupported_command(args.first()));
    }
    Ok(false)
}

fn bind_attached_session(command: &TmuxCommand, args: &mut Vec<String>, session: &str) {
    if has_explicit_target(args)
        || matches!(
            command,
            TmuxCommand::NewSession
                | TmuxCommand::AttachSession
                | TmuxCommand::ListSessions
                | TmuxCommand::ListClients
                | TmuxCommand::KillServer
                | TmuxCommand::Other
        )
    {
        return;
    }
    let command_index = args
        .iter()
        .position(|arg| !matches!(arg.as_str(), "-d" | "--detached"))
        .unwrap_or(0);
    args.splice(
        command_index.saturating_add(1)..command_index.saturating_add(1),
        [String::from("-t"), session.to_string()],
    );
}

fn has_explicit_target(args: &[String]) -> bool {
    args.iter()
        .any(|arg| arg == "-t" || (arg.starts_with("-t") && arg.len() > 2))
}

fn current_session_name(endpoint: &TmuxIpcEndpoint) -> io::Result<String> {
    match request_endpoint_response(endpoint, TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info { session_name, .. } => Ok(session_name),
        TmuxIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, reason))
        }
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")),
        )),
    }
}

fn send_detach(endpoint: &TmuxIpcEndpoint, client_id: &str) -> io::Result<()> {
    match request_endpoint_response(
        endpoint,
        TmuxIpcRequest::DetachClient {
            client_id: client_id.to_string(),
        },
    )? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, reason))
        }
        _ => Ok(()),
    }
}

fn unsupported_command(command: Option<&String>) -> io::Error {
    let command = command.map(String::as_str).unwrap_or("unknown");
    io::Error::new(
        io::ErrorKind::Unsupported,
        terman_common::builtin_tmux_command_unsupported_hint(command),
    )
}

fn split_command_line(line: &str) -> Result<Vec<String>, ()> {
    let mut args = Vec::new();
    let mut token = String::new();
    let mut quote = None;
    let mut escaped = false;
    let mut started = false;
    for ch in line.chars() {
        if escaped {
            token.push(ch);
            escaped = false;
            started = true;
            continue;
        }
        match quote {
            Some('\'') if ch == '\'' => quote = None,
            Some('"') if ch == '"' => quote = None,
            Some('"') if ch == '\\' => escaped = true,
            Some(_) => {
                token.push(ch);
                started = true;
            }
            None if ch == '\'' || ch == '"' => {
                quote = Some(ch);
                started = true;
            }
            None if ch == '\\' => {
                escaped = true;
                started = true;
            }
            None if ch.is_whitespace() => {
                if started {
                    args.push(std::mem::take(&mut token));
                    started = false;
                }
            }
            None => {
                token.push(ch);
                started = true;
            }
        }
    }
    if escaped || quote.is_some() {
        return Err(());
    }
    if started {
        args.push(token);
    }
    Ok(args)
}
