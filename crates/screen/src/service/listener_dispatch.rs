use std::{
    io::{self, BufRead, BufReader},
    sync::{Arc, Mutex, mpsc},
};

use interprocess::local_socket::prelude::*;

use super::listener_io::{send_control_event, stream_attach, write_response, write_result_response};
use crate::{
    ipc::{ScreenIpcRequest, ScreenIpcResponse, ScreenWindowInfo},
    session_core::{ScreenControlEvent, ScreenSessionBus},
};

pub(super) fn handle_client(
    stream: &mut LocalSocketStream,
    session_name: &Arc<Mutex<String>>,
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
            client_id,
            detach_existing,
            ..
        }) => {
            if detach_existing {
                bus.publish_detach();
            }
            stream_attach(stream, bus, client_id)
        }
        Ok(ScreenIpcRequest::Detach) => write_response(stream, &ScreenIpcResponse::Accepted),
        Ok(ScreenIpcRequest::DetachClient { client_id }) => {
            bus.detach_client(&client_id);
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::DetachAll) => {
            bus.publish_detach();
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::Bell) => {
            bus.publish_transient_output(b"\x07");
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::Clear) => {
            bus.clear_replay();
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::Echo { message }) => {
            let mut bytes = message.into_bytes();
            bytes.extend_from_slice(b"\r\n");
            bus.publish_message(&bytes);
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::SetHardcopyDir { path }) => {
            bus.set_hardcopy_dir(path);
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::SetHardcopyAppend { append }) => {
            bus.set_hardcopy_append(append);
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::SetLogEnabled { enabled }) => {
            write_result_response(stream, bus.set_log_enabled(enabled))
        }
        Ok(ScreenIpcRequest::ToggleLog) => write_result_response(stream, bus.toggle_log_enabled()),
        Ok(ScreenIpcRequest::SetLogFile { path }) => {
            write_result_response(stream, bus.set_log_path(path))
        }
        Ok(ScreenIpcRequest::SetLogFlush { seconds }) => {
            write_result_response(stream, bus.set_log_flush_interval(seconds))
        }
        Ok(ScreenIpcRequest::SetLogTimestampEnabled { enabled }) => {
            write_result_response(stream, bus.set_log_timestamp_enabled(enabled))
        }
        Ok(ScreenIpcRequest::ToggleLogTimestamp) => {
            write_result_response(stream, bus.toggle_log_timestamp_enabled())
        }
        Ok(ScreenIpcRequest::SetLogTimestampAfter { seconds }) => {
            write_result_response(stream, bus.set_log_timestamp_after(seconds))
        }
        Ok(ScreenIpcRequest::SetLogTimestampString { value }) => {
            write_result_response(stream, bus.set_log_timestamp_format(value))
        }
        Ok(ScreenIpcRequest::SetPasteBuffer { bytes }) => {
            bus.set_paste_buffer(bytes);
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::SetRegister { name, bytes }) => {
            bus.set_register(name, bytes);
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::PasteRegister { name }) => {
            send_control_event(control_tx, ScreenControlEvent::Input(bus.register_snapshot(&name)))?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::SetScrollback { lines }) => {
            bus.set_scrollback_lines(lines);
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::SetDefaultScrollback { lines }) => {
            send_control_event(control_tx, ScreenControlEvent::SetDefaultScrollback { lines })?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::SetWindowTitle { title }) => {
            bus.set_window_title(title);
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
        Ok(ScreenIpcRequest::NewWindow { command }) => {
            send_control_event(control_tx, ScreenControlEvent::NewWindow { command })?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::SetDefaultCwd { path }) => {
            send_control_event(control_tx, ScreenControlEvent::SetDefaultCwd { path })?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::SetEnv { name, value }) => {
            send_control_event(control_tx, ScreenControlEvent::SetEnv { name, value })?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::UnsetEnv { name }) => {
            send_control_event(control_tx, ScreenControlEvent::UnsetEnv { name })?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::GetPasteBuffer) => write_response(
            stream,
            &ScreenIpcResponse::PasteBuffer {
                bytes: bus.paste_buffer_snapshot(),
            },
        ),
        Ok(ScreenIpcRequest::LastMessage) => write_last_message(stream, bus),
        Ok(ScreenIpcRequest::Info) => write_info(stream, session_name, bus),
        Ok(ScreenIpcRequest::PasteBuffer) => {
            send_control_event(control_tx, ScreenControlEvent::Input(bus.paste_buffer_snapshot()))?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::SelectWindow { index }) => {
            send_control_event(control_tx, ScreenControlEvent::SelectWindow { index })?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::NumberWindow { source, index }) => {
            send_control_event(control_tx, ScreenControlEvent::NumberWindow { source, index })?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::NextWindow) => {
            send_control_event(control_tx, ScreenControlEvent::NextWindow)?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::PreviousWindow) => {
            send_control_event(control_tx, ScreenControlEvent::PreviousWindow)?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::LastWindow) => {
            send_control_event(control_tx, ScreenControlEvent::LastWindow)?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::KillWindow) => {
            send_control_event(control_tx, ScreenControlEvent::KillWindow)?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::Ping) => write_response(stream, &ScreenIpcResponse::Accepted),
        Ok(ScreenIpcRequest::Quit) => {
            send_control_event(control_tx, ScreenControlEvent::Terminate)?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::RenameSession { name }) => match rename_session(session_name, name) {
            Ok(()) => write_response(stream, &ScreenIpcResponse::Accepted),
            Err(err) => write_response(
                stream,
                &ScreenIpcResponse::Rejected {
                    reason: err.to_string(),
                },
            ),
        },
        Ok(ScreenIpcRequest::Input { bytes }) => {
            send_control_event(control_tx, ScreenControlEvent::Input(bytes))?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::Resize { cols, rows }) => {
            send_control_event(control_tx, ScreenControlEvent::Resize { cols, rows })?;
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

fn write_last_message(stream: &mut LocalSocketStream, bus: &ScreenSessionBus) -> io::Result<()> {
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

fn write_info(
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
            window_title: status.window_title,
            active_window: status.active_window,
            windows,
        },
    )
}

fn current_session_name(session_name: &Arc<Mutex<String>>) -> io::Result<String> {
    session_name
        .lock()
        .map(|name| name.clone())
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))
}

fn rename_session(session_name: &Arc<Mutex<String>>, name: String) -> io::Result<()> {
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