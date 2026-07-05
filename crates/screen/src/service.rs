use std::{
    io::{self, BufRead, BufReader, Write},
    thread,
};

use interprocess::local_socket::prelude::*;

use crate::{
    ScreenArgs,
    ipc::{ScreenAttachMode, ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse},
    session_core::{ScreenSessionBus, ScreenSessionEvent},
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
                let client_bus = bus.clone();
                thread::spawn(move || {
                    let _ = handle_client(&mut stream, &client_bus);
                });
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

    read_attach_stream(stream)
}

fn read_attach_stream(stream: LocalSocketStream) -> io::Result<()> {
    let mut reader = BufReader::new(stream);
    let mut stdout = io::stdout();

    loop {
        let mut response = String::new();
        if reader.read_line(&mut response)? == 0 {
            return Ok(());
        }
        let response: ScreenIpcResponse = serde_json::from_str(response.trim_end())
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;

        match response {
            ScreenIpcResponse::Accepted => {}
            ScreenIpcResponse::Attached { replay } => {
                stdout.write_all(&replay)?;
                stdout.flush()?;
            }
            ScreenIpcResponse::Output { bytes } => {
                stdout.write_all(&bytes)?;
                stdout.flush()?;
            }
            ScreenIpcResponse::Resize { .. } => {}
            ScreenIpcResponse::Exit { .. } => return Ok(()),
            ScreenIpcResponse::Rejected { reason } => {
                return Err(io::Error::new(io::ErrorKind::Unsupported, reason));
            }
        }
    }
}

fn handle_client(stream: &mut LocalSocketStream, bus: &ScreenSessionBus) -> io::Result<()> {
    let mut request = String::new();
    {
        let mut reader = BufReader::new(&mut *stream);
        reader.read_line(&mut request)?;
    }

    match serde_json::from_str::<ScreenIpcRequest>(request.trim_end()) {
        Ok(ScreenIpcRequest::Attach { .. }) => stream_attach(stream, bus),
        Ok(ScreenIpcRequest::Detach) => write_response(stream, &ScreenIpcResponse::Accepted),
        Ok(ScreenIpcRequest::Input { .. } | ScreenIpcRequest::Resize { .. }) => write_response(
            stream,
            &ScreenIpcResponse::Rejected {
                reason: terman_common::builtin_screen_attach_unsupported_hint(),
            },
        ),
        Err(err) => write_response(
            stream,
            &ScreenIpcResponse::Rejected {
                reason: err.to_string(),
            },
        ),
    }
}

fn stream_attach(stream: &mut LocalSocketStream, bus: &ScreenSessionBus) -> io::Result<()> {
    let (replay, events) = bus.subscribe_with_replay();
    write_response(stream, &ScreenIpcResponse::Attached { replay })?;

    for event in events {
        let response = match event {
            ScreenSessionEvent::Output(bytes) => ScreenIpcResponse::Output { bytes },
            ScreenSessionEvent::Resize { cols, rows } => ScreenIpcResponse::Resize { cols, rows },
            ScreenSessionEvent::Exit(code) => ScreenIpcResponse::Exit { code },
        };
        let should_close = matches!(response, ScreenIpcResponse::Exit { .. });
        write_response(stream, &response)?;
        if should_close {
            break;
        }
    }

    Ok(())
}

fn write_response(stream: &mut LocalSocketStream, response: &ScreenIpcResponse) -> io::Result<()> {
    serde_json::to_writer(&mut *stream, response)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    stream.write_all(b"\n")?;
    stream.flush()
}