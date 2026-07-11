use std::{io, sync::mpsc};

use interprocess::local_socket::prelude::*;

use crate::{
    ipc::{TmuxIpcResponse, TmuxBufferInfo},
    service_codec::write_response,
    session_core::{TmuxControlEvent, TmuxSessionBus},
};

pub(crate) fn get_buffer(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    name: Option<String>,
) -> io::Result<()> {
    match bus.buffer_snapshot(name.as_deref()) {
        Some(TmuxBufferInfo { name, bytes }) => {
            write_response(stream, &TmuxIpcResponse::Buffer { name, bytes })
        }
        None => write_missing(stream, name.as_deref()),
    }
}

pub(crate) fn list_buffers(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
) -> io::Result<()> {
    write_response(
        stream,
        &TmuxIpcResponse::Buffers {
            buffers: bus.buffers_snapshot(),
        },
    )
}

pub(crate) fn set_buffer(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    name: Option<String>,
    bytes: Vec<u8>,
) -> io::Result<()> {
    if bus.set_buffer(name, bytes) {
        write_response(stream, &TmuxIpcResponse::Accepted)
    } else {
        write_unavailable(stream)
    }
}

pub(crate) fn delete_buffer(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    name: Option<String>,
) -> io::Result<()> {
    if bus.delete_buffer(name.as_deref()) {
        write_response(stream, &TmuxIpcResponse::Accepted)
    } else {
        write_missing(stream, name.as_deref())
    }
}

pub(crate) fn paste_buffer(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    control_tx: &mpsc::Sender<TmuxControlEvent>,
    name: Option<String>,
) -> io::Result<()> {
    let Some(buffer) = bus.buffer_snapshot(name.as_deref()) else {
        return write_missing(stream, name.as_deref());
    };
    control_tx
        .send(TmuxControlEvent::Input(buffer.bytes))
        .map_err(|error| io::Error::new(io::ErrorKind::BrokenPipe, error.to_string()))?;
    write_response(stream, &TmuxIpcResponse::Accepted)
}

fn write_missing(
    stream: &mut LocalSocketStream,
    name: Option<&str>,
) -> io::Result<()> {
    write_response(
        stream,
        &TmuxIpcResponse::Rejected {
            reason: terman_common::builtin_tmux_buffer_not_found_hint(
                name.unwrap_or("top"),
            ),
        },
    )
}

fn write_unavailable(stream: &mut LocalSocketStream) -> io::Result<()> {
    write_response(
        stream,
        &TmuxIpcResponse::Rejected {
            reason: terman_common::builtin_tmux_buffer_unavailable_hint(),
        },
    )
}