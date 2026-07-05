use std::{
    io::{self, BufRead, BufReader, Write},
    thread,
};

use interprocess::local_socket::prelude::*;

use crate::{
    ScreenArgs,
    ipc::{ScreenAttachMode, ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse},
    session_core::ScreenSessionBus,
    sessions::find_builtin_screen_session_for_attach,
};

pub(crate) struct ScreenSessionService {
    _handle: thread::JoinHandle<()>,
}

impl ScreenSessionService {
    pub(crate) fn start(
        session_name: Option<&str>,
        bus: ScreenSessionBus,
    ) -> io::Result<Option<Self>> {
        let Some(session_name) = session_name else {
            return Ok(None);
        };

        let endpoint = ScreenIpcEndpoint::for_session(session_name);
        let listener = endpoint.listener_options()?.create_sync()?;
        let handle = thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else {
                    continue;
                };
                let _ = handle_client(&mut stream, &bus);
            }
        });

        Ok(Some(Self { _handle: handle }))
    }
}

pub(crate) fn request_screen_attach(args: &ScreenArgs) -> io::Result<()> {
    let (mode, target) = match (&args.resume, &args.multi_attach) {
        (Some(target), None) => (ScreenAttachMode::Resume, target.as_deref()),
        (None, Some(target)) => (ScreenAttachMode::MultiAttach, target.as_deref()),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                terman_common::builtin_screen_attach_target_required_hint(),
            ));
        }
    };

    let session = find_builtin_screen_session_for_attach(target)?;
    let endpoint = session
        .ipc_endpoint
        .as_deref()
        .map(ScreenIpcEndpoint::from_raw_name)
        .unwrap_or_else(|| ScreenIpcEndpoint::for_session(&session.name));
    let mut stream = endpoint.connect_options()?.connect_sync()?;
    let request = ScreenIpcRequest::Attach {
        mode,
        target: Some(session.name),
    };

    serde_json::to_writer(&mut stream, &request)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut response = String::new();
    BufReader::new(stream).read_line(&mut response)?;
    let response: ScreenIpcResponse = serde_json::from_str(response.trim_end())
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

    match response {
        ScreenIpcResponse::Accepted => Ok(()),
        ScreenIpcResponse::Attached { replay } => {
            let mut stdout = io::stdout();
            stdout.write_all(&replay)?;
            stdout.flush()
        }
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
    }
}

fn handle_client(stream: &mut LocalSocketStream, bus: &ScreenSessionBus) -> io::Result<()> {
    let mut request = String::new();
    {
        let mut reader = BufReader::new(&mut *stream);
        reader.read_line(&mut request)?;
    }

    let response = match serde_json::from_str::<ScreenIpcRequest>(request.trim_end()) {
        Ok(ScreenIpcRequest::Attach { .. }) => ScreenIpcResponse::Attached {
            replay: bus.replay_snapshot(),
        },
        Ok(ScreenIpcRequest::Detach) => ScreenIpcResponse::Accepted,
        Ok(ScreenIpcRequest::Input { .. } | ScreenIpcRequest::Resize { .. }) => {
            ScreenIpcResponse::Rejected {
                reason: terman_common::builtin_screen_attach_unsupported_hint(),
            }
        }
        Err(err) => ScreenIpcResponse::Rejected {
            reason: err.to_string(),
        },
    };

    serde_json::to_writer(&mut *stream, &response)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    stream.write_all(b"\n")?;
    stream.flush()
}