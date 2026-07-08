use std::{io, sync::{Arc, Mutex}};

use interprocess::local_socket::prelude::*;

use super::listener_io::write_response;
use crate::{
    ipc::{ScreenIpcResponse, ScreenWindowInfo},
    session_core::ScreenSessionBus,
};

pub(super) fn write_last_message(
    stream: &mut LocalSocketStream,
    bus: &ScreenSessionBus,
) -> io::Result<()> {
    let mut bytes = bus.last_message_snapshot();
    if bytes.is_empty() {
        bytes = terman_common::builtin_screen_control_lastmsg_empty_hint().into_bytes();
        bytes.extend_from_slice(b"\r\n");
        bus.publish_message(&bytes);
    } else {
        bus.publish_transient_output(&bytes);
    }
    write_response(stream, &ScreenIpcResponse::Accepted)
}

pub(super) fn write_info(
    stream: &mut LocalSocketStream,
    session_name: &Arc<Mutex<String>>,
    bus: &ScreenSessionBus,
) -> io::Result<()> {
    let status = bus.status_snapshot();
    let session_name = current_session_name(session_name)?;
    let windows = status
        .windows
        .into_iter()
        .map(|window| ScreenWindowInfo {
            index: window.index,
            title: window.title.unwrap_or_else(|| session_name.clone()),
            active: window.active,
            replay_bytes: window.replay_bytes,
        })
        .collect();
    write_response(
        stream,
        &ScreenIpcResponse::Info {
            session_name,
            replay_bytes: status.replay_bytes,
            attach_clients: status.attach_clients,
            cols: status.cols,
            rows: status.rows,
            scrollback_lines: status.scrollback_lines,
            hardcopy_dir: status.hardcopy_dir,
            hardcopy_append: status.hardcopy_append,
            buffer_file: status.buffer_file,
            window_title: status.window_title,
            active_window: status.active_window,
            windows,
        },
    )
}

pub(super) fn rename_session(session_name: &Arc<Mutex<String>>, name: String) -> io::Result<()> {
    if name.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_session_name_empty_hint(),
        ));
    }
    let mut session_name = session_name
        .lock()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;
    *session_name = name;
    Ok(())
}

fn current_session_name(session_name: &Arc<Mutex<String>>) -> io::Result<String> {
    session_name
        .lock()
        .map(|name| name.clone())
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))
}
