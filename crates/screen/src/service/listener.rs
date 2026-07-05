use std::{
    io::{self, BufRead, BufReader, Write},
    sync::mpsc,
    thread,
};

use interprocess::local_socket::prelude::*;

use crate::{
    ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse},
    session_core::{ScreenControlEvent, ScreenSessionBus, ScreenSessionEvent},
};

pub(crate) struct ScreenSessionService {
    _handle: thread::JoinHandle<()>,
}

impl ScreenSessionService {
    pub(crate) fn start(
        session_name: Option<&str>,
        bus: ScreenSessionBus,
        control_tx: mpsc::Sender<ScreenControlEvent>,
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
                let client_control_tx = control_tx.clone();
                thread::spawn(move || {
                    let _ = handle_client(&mut stream, &client_bus, &client_control_tx);
                });
            }
        });

        Ok(Some(Self { _handle: handle }))
    }
}

fn handle_client(
    stream: &mut LocalSocketStream,
    bus: &ScreenSessionBus,
    control_tx: &mpsc::Sender<ScreenControlEvent>,
) -> io::Result<()> {
    let mut request = String::new();
    {
        let mut reader = BufReader::new(&mut *stream);
        reader.read_line(&mut request)?;
    }

    match serde_json::from_str::<ScreenIpcRequest>(request.trim_end()) {
        Ok(ScreenIpcRequest::Attach {
            detach_existing, ..
        }) => {
            if detach_existing {
                bus.publish_detach();
            }
            stream_attach(stream, bus)
        }
        Ok(ScreenIpcRequest::Detach) => write_response(stream, &ScreenIpcResponse::Accepted),
        Ok(ScreenIpcRequest::DetachAll) => {
            bus.publish_detach();
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::Clear) => {
            bus.clear_replay();
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::Reset) => {
            bus.clear_replay();
            bus.publish_transient_output(b"\x1bc");
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::Hardcopy) => write_response(
            stream,
            &ScreenIpcResponse::Hardcopy {
                bytes: bus.replay_snapshot(),
            },
        ),
        Ok(ScreenIpcRequest::Info) => {
            let status = bus.status_snapshot();
            write_response(
                stream,
                &ScreenIpcResponse::Info {
                    replay_bytes: status.replay_bytes,
                    attach_clients: status.attach_clients,
                },
            )
        }
        Ok(ScreenIpcRequest::Ping) => write_response(stream, &ScreenIpcResponse::Accepted),
        Ok(ScreenIpcRequest::Quit) => {
            control_tx
                .send(ScreenControlEvent::Terminate)
                .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err.to_string()))?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::Input { bytes }) => {
            control_tx
                .send(ScreenControlEvent::Input(bytes))
                .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err.to_string()))?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::Resize { cols, rows }) => {
            control_tx
                .send(ScreenControlEvent::Resize { cols, rows })
                .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err.to_string()))?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
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

    while let Ok(event) = events.recv() {
        let response = match event {
            ScreenSessionEvent::Output(bytes) => ScreenIpcResponse::Output { bytes },
            ScreenSessionEvent::Resize { cols, rows } => ScreenIpcResponse::Resize { cols, rows },
            ScreenSessionEvent::Detach => ScreenIpcResponse::Detached,
            ScreenSessionEvent::Exit(code) => ScreenIpcResponse::Exit { code },
        };
        let should_close = matches!(
            response,
            ScreenIpcResponse::Detached | ScreenIpcResponse::Exit { .. }
        );
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


