use std::{
    error::Error,
    io::{self, Write},
};

use crate::{
    buffer_args::{buffer_data_arg, buffer_name_arg},
    command::TmuxCommand,
    ipc::{TmuxBufferInfo, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
    sessions::{BuiltinTmuxSession, load_builtin_tmux_sessions},
    window_command_support::{
        required_target_session_name_arg, session_endpoint, session_not_found_error,
        unexpected_response_error,
    },
};

pub(crate) fn run_builtin_tmux_buffer_command(
    command: &TmuxCommand,
    args: &[String],
) -> Result<(), Box<dyn Error>> {
    let target = required_target_session_name_arg(args)?;
    let session = find_session(&target)?;
    let name = buffer_name_arg(args);
    match command {
        TmuxCommand::SetBuffer => set_buffer(&session, name, args),
        TmuxCommand::ShowBuffer => show_buffer(&session, name),
        TmuxCommand::ListBuffers => list_buffers(&session),
        TmuxCommand::PasteBuffer => request_accepted(
            &session,
            TmuxIpcRequest::PasteBuffer { name },
        ),
        TmuxCommand::DeleteBuffer => request_accepted(
            &session,
            TmuxIpcRequest::DeleteBuffer { name },
        ),
        _ => Ok(()),
    }
}

fn set_buffer(
    session: &BuiltinTmuxSession,
    name: Option<String>,
    args: &[String],
) -> Result<(), Box<dyn Error>> {
    let data = buffer_data_arg(args).ok_or_else(buffer_data_required_error)?;
    request_accepted(
        session,
        TmuxIpcRequest::SetBuffer {
            name,
            bytes: data.into_bytes(),
        },
    )
}

fn show_buffer(
    session: &BuiltinTmuxSession,
    name: Option<String>,
) -> Result<(), Box<dyn Error>> {
    match request_endpoint_response(
        &session_endpoint(session),
        TmuxIpcRequest::GetBuffer { name },
    )? {
        TmuxIpcResponse::Buffer { bytes, .. } => write_buffer(&bytes),
        TmuxIpcResponse::Rejected { reason } => Err(rejected_error(reason)),
        response => Err(unexpected_response_error(response)),
    }
}

fn list_buffers(session: &BuiltinTmuxSession) -> Result<(), Box<dyn Error>> {
    match request_endpoint_response(
        &session_endpoint(session),
        TmuxIpcRequest::ListBuffers,
    )? {
        TmuxIpcResponse::Buffers { buffers } => {
            for buffer in buffers {
                println!(
                    "{}",
                    terman_common::builtin_tmux_buffer_list_item_hint(
                        &buffer.name,
                        buffer.bytes.len(),
                        &buffer_preview(&buffer),
                    )
                );
            }
            Ok(())
        }
        TmuxIpcResponse::Rejected { reason } => Err(rejected_error(reason)),
        response => Err(unexpected_response_error(response)),
    }
}

fn request_accepted(
    session: &BuiltinTmuxSession,
    request: TmuxIpcRequest,
) -> Result<(), Box<dyn Error>> {
    match request_endpoint_response(&session_endpoint(session), request)? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => Err(rejected_error(reason)),
        response => Err(unexpected_response_error(response)),
    }
}

fn find_session(target: &str) -> Result<BuiltinTmuxSession, Box<dyn Error>> {
    load_builtin_tmux_sessions()?
        .into_iter()
        .find(|session| session.name == target)
        .ok_or_else(|| session_not_found_error(target))
}

fn write_buffer(bytes: &[u8]) -> Result<(), Box<dyn Error>> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(bytes)?;
    if !bytes.ends_with(b"\n") {
        stdout.write_all(b"\n")?;
    }
    stdout.flush()?;
    Ok(())
}

fn buffer_preview(buffer: &TmuxBufferInfo) -> String {
    let text = String::from_utf8_lossy(&buffer.bytes)
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .collect::<String>();
    terman_common::truncate_terminal_text(text.trim(), 48)
}

fn buffer_data_required_error() -> Box<dyn Error> {
    Box::new(io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_tmux_buffer_data_required_hint(),
    ))
}

fn rejected_error(reason: String) -> Box<dyn Error> {
    Box::new(io::Error::new(io::ErrorKind::Unsupported, reason))
}