use std::{
    io::{self, BufRead, BufReader, Write},
    sync::mpsc,
    thread,
};

use interprocess::local_socket::prelude::*;

use crate::{
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    session_core::{TmuxControlEvent, TmuxSessionBus, TmuxSessionEvent},
};

#[allow(dead_code)]
pub(crate) struct TmuxSessionService {
    _handle: thread::JoinHandle<()>,
}

impl TmuxSessionService {
    #[allow(dead_code)]
    pub(crate) fn start(
        session_name: &str,
        endpoint: TmuxIpcEndpoint,
        cwd: String,
        bus: TmuxSessionBus,
        control_tx: mpsc::Sender<TmuxControlEvent>,
    ) -> io::Result<Self> {
        let listener = endpoint.listener_options()?.create_sync()?;
        let session_name = session_name.to_string();
        let handle = thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else {
                    continue;
                };
                let client_bus = bus.clone();
                let client_control_tx = control_tx.clone();
                let client_session_name = session_name.clone();
                let client_cwd = cwd.clone();
                thread::spawn(move || {
                    let _ = handle_client(
                        &mut stream,
                        &client_session_name,
                        &client_cwd,
                        &client_bus,
                        &client_control_tx,
                    );
                });
            }
        });

        Ok(Self { _handle: handle })
    }
}

#[allow(dead_code)]
pub(crate) fn request_endpoint_response(
    endpoint: &TmuxIpcEndpoint,
    request: TmuxIpcRequest,
) -> io::Result<TmuxIpcResponse> {
    let mut stream = endpoint.connect_options()?.connect_sync()?;
    write_request(&mut stream, &request)?;
    read_response(stream)
}

fn handle_client(
    stream: &mut LocalSocketStream,
    session_name: &str,
    cwd: &str,
    bus: &TmuxSessionBus,
    control_tx: &mpsc::Sender<TmuxControlEvent>,
) -> io::Result<()> {
    let mut request = String::new();
    {
        let mut reader = BufReader::new(&mut *stream);
        reader.read_line(&mut request)?;
    }

    match serde_json::from_str::<TmuxIpcRequest>(request.trim_end()) {
        Ok(TmuxIpcRequest::Attach { client_id }) => stream_attach(stream, bus, client_id),
        Ok(TmuxIpcRequest::Detach) => write_response(stream, &TmuxIpcResponse::Accepted),
        Ok(TmuxIpcRequest::DetachClient { client_id }) => {
            bus.detach_client(&client_id);
            write_response(stream, &TmuxIpcResponse::Accepted)
        }
        Ok(TmuxIpcRequest::DetachAll) => {
            bus.publish_detach();
            write_response(stream, &TmuxIpcResponse::Accepted)
        }
        Ok(TmuxIpcRequest::Info) => {
            let status = bus.status_snapshot();
            write_response(
                stream,
                &TmuxIpcResponse::Info {
                    session_name: session_name.to_string(),
                    windows: status.windows,
                    attached_clients: status.attached_clients,
                    cwd: cwd.to_string(),
                },
            )
        }
        Ok(TmuxIpcRequest::Input { bytes }) => {
            send_control(control_tx, TmuxControlEvent::Input(bytes))?;
            write_response(stream, &TmuxIpcResponse::Accepted)
        }
        Ok(TmuxIpcRequest::Ping) => write_response(stream, &TmuxIpcResponse::Accepted),
        Ok(TmuxIpcRequest::Quit) => {
            send_control(control_tx, TmuxControlEvent::Terminate)?;
            write_response(stream, &TmuxIpcResponse::Accepted)
        }
        Ok(TmuxIpcRequest::Resize { cols, rows }) => {
            send_control(control_tx, TmuxControlEvent::Resize { cols, rows })?;
            write_response(stream, &TmuxIpcResponse::Accepted)
        }
        Err(err) => write_response(
            stream,
            &TmuxIpcResponse::Rejected {
                reason: err.to_string(),
            },
        ),
    }
}

fn stream_attach(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    client_id: Option<String>,
) -> io::Result<()> {
    let (replay, events) = bus.subscribe_with_replay(client_id);
    write_response(stream, &TmuxIpcResponse::Attached { replay })?;

    while let Ok(event) = events.recv() {
        let response = match event {
            TmuxSessionEvent::Output(bytes) => TmuxIpcResponse::Output { bytes },
            TmuxSessionEvent::Resize { cols, rows } => TmuxIpcResponse::Resize { cols, rows },
            TmuxSessionEvent::Detach => TmuxIpcResponse::Detached,
            TmuxSessionEvent::Exit(code) => TmuxIpcResponse::Exit { code },
        };
        let should_close = matches!(
            response,
            TmuxIpcResponse::Detached | TmuxIpcResponse::Exit { .. }
        );
        write_response(stream, &response)?;
        if should_close {
            break;
        }
    }

    Ok(())
}

fn send_control(
    control_tx: &mpsc::Sender<TmuxControlEvent>,
    event: TmuxControlEvent,
) -> io::Result<()> {
    control_tx
        .send(event)
        .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err.to_string()))
}

fn write_request(stream: &mut LocalSocketStream, request: &TmuxIpcRequest) -> io::Result<()> {
    serde_json::to_writer(&mut *stream, request)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    stream.write_all(b"\n")?;
    stream.flush()
}

fn read_response(stream: LocalSocketStream) -> io::Result<TmuxIpcResponse> {
    let mut response = String::new();
    let mut reader = BufReader::new(stream);
    reader.read_line(&mut response)?;
    serde_json::from_str(response.trim_end())
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

fn write_response(stream: &mut LocalSocketStream, response: &TmuxIpcResponse) -> io::Result<()> {
    serde_json::to_writer(&mut *stream, response)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    stream.write_all(b"\n")?;
    stream.flush()
}

#[cfg(test)]
mod tests {
    use crate::ipc::TmuxIpcRequest;

    #[test]
    fn models_client_request_payload() {
        assert_eq!(TmuxIpcRequest::Ping, TmuxIpcRequest::Ping);
    }
}