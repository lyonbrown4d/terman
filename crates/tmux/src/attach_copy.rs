use std::io::{self, Write};

use crate::{
    attach_status::{query_status_line, render_status_line},
    copy_mode::TmuxCopyMode,
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
};

pub(crate) fn start_attach_copy_mode(
    endpoint: &TmuxIpcEndpoint,
) -> io::Result<TmuxCopyMode> {
    let replay = match request_endpoint_response(
        endpoint,
        TmuxIpcRequest::CapturePane {
            window: None,
            pane: None,
        },
    )? {
        TmuxIpcResponse::Captured { bytes } => bytes,
        TmuxIpcResponse::Rejected { reason } => {
            return Err(io::Error::new(io::ErrorKind::Unsupported, reason));
        }
        response => return Err(unexpected_response(response)),
    };
    let (cols, rows) = terman_common::current_terminal_size()?;
    Ok(TmuxCopyMode::from_replay(
        &replay,
        cols,
        terman_common::terminal_rows_without_status(rows),
    ))
}

pub(crate) fn finish_attach_copy_mode(
    endpoint: &TmuxIpcEndpoint,
    copied: Option<Vec<u8>>,
) -> io::Result<()> {
    if let Some(bytes) = copied {
        expect_accepted(request_endpoint_response(
            endpoint,
            TmuxIpcRequest::SetBuffer { bytes },
        )?)?;
    }
    let replay = match request_endpoint_response(endpoint, TmuxIpcRequest::RefreshClient)? {
        TmuxIpcResponse::Captured { bytes } => bytes,
        TmuxIpcResponse::Rejected { reason } => {
            return Err(io::Error::new(io::ErrorKind::Unsupported, reason));
        }
        response => return Err(unexpected_response(response)),
    };
    let mut stdout = io::stdout().lock();
    stdout.write_all(b"\x1bc")?;
    stdout.write_all(&replay)?;
    stdout.flush()?;
    drop(stdout);
    render_status_line(&query_status_line(endpoint)?)
}

fn expect_accepted(response: TmuxIpcResponse) -> io::Result<()> {
    match response {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, reason))
        }
        response => Err(unexpected_response(response)),
    }
}

fn unexpected_response(response: TmuxIpcResponse) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")),
    )
}