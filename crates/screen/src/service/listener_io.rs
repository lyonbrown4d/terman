use std::{
    io::{self, Write},
    sync::mpsc,
};

use interprocess::local_socket::prelude::*;

use crate::{
    ipc::ScreenIpcResponse,
    session_core::{ScreenControlEvent, ScreenSessionBus, ScreenSessionEvent},
};

pub(super) fn stream_attach(
    stream: &mut LocalSocketStream,
    bus: &ScreenSessionBus,
    client_id: Option<String>,
) -> io::Result<()> {
    let (replay, events) = bus.subscribe_with_replay(client_id);
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

pub(super) fn send_control_event(
    control_tx: &mpsc::Sender<ScreenControlEvent>,
    event: ScreenControlEvent,
) -> io::Result<()> {
    control_tx
        .send(event)
        .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err.to_string()))
}

pub(super) fn write_result_response(
    stream: &mut LocalSocketStream,
    result: io::Result<()>,
) -> io::Result<()> {
    match result {
        Ok(()) => write_response(stream, &ScreenIpcResponse::Accepted),
        Err(err) => write_response(
            stream,
            &ScreenIpcResponse::Rejected {
                reason: err.to_string(),
            },
        ),
    }
}

pub(super) fn write_response(
    stream: &mut LocalSocketStream,
    response: &ScreenIpcResponse,
) -> io::Result<()> {
    serde_json::to_writer(&mut *stream, response)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    stream.write_all(b"\n")?;
    stream.flush()
}