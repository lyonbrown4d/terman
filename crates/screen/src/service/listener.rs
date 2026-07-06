use std::{
    io::{self, BufRead, BufReader, Write},
    sync::{Arc, Mutex, mpsc},
    thread,
};

use interprocess::local_socket::prelude::*;

use crate::{
    ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse, ScreenWindowInfo},
    session_core::{ScreenControlEvent, ScreenSessionBus, ScreenSessionEvent},
};

pub(crate) struct ScreenSessionService {
    _handle: thread::JoinHandle<()>,
}

impl ScreenSessionService {
    pub(crate) fn start(
        session_name: Option<Arc<Mutex<String>>>,
        endpoint: ScreenIpcEndpoint,
        bus: ScreenSessionBus,
        control_tx: mpsc::Sender<ScreenControlEvent>,
    ) -> io::Result<Option<Self>> {
        let Some(session_name) = session_name else {
            return Ok(None);
        };

        let listener = endpoint.listener_options()?.create_sync()?;
        let handle = thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else {
                    continue;
                };
                let client_bus = bus.clone();
                let client_control_tx = control_tx.clone();
                let client_session_name = session_name.clone();
                thread::spawn(move || {
                    let _ = handle_client(
                        &mut stream,
                        &client_session_name,
                        &client_bus,
                        &client_control_tx,
                    );
                });
            }
        });

        Ok(Some(Self { _handle: handle }))
    }
}

fn handle_client(
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
            bus.publish_transient_output(&bytes);
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::SetLogEnabled { enabled }) => {
            write_result_response(stream, bus.set_log_enabled(enabled))
        }
        Ok(ScreenIpcRequest::SetLogFile { path }) => {
            write_result_response(stream, bus.set_log_path(path))
        }
        Ok(ScreenIpcRequest::SetPasteBuffer { bytes }) => {
            bus.set_paste_buffer(bytes);
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::SetScrollback { lines }) => {
            bus.set_scrollback_lines(lines);
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
            control_tx
                .send(ScreenControlEvent::NewWindow { command })
                .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err.to_string()))?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::GetPasteBuffer) => write_response(
            stream,
            &ScreenIpcResponse::PasteBuffer {
                bytes: bus.paste_buffer_snapshot(),
            },
        ),
        Ok(ScreenIpcRequest::Info) => {
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
                    window_title: status.window_title,
                    active_window: status.active_window,
                    windows,
                },
            )
        }
        Ok(ScreenIpcRequest::PasteBuffer) => {
            control_tx
                .send(ScreenControlEvent::Input(bus.paste_buffer_snapshot()))
                .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err.to_string()))?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::SelectWindow { index }) => {
            control_tx
                .send(ScreenControlEvent::SelectWindow { index })
                .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err.to_string()))?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::NextWindow) => {
            control_tx
                .send(ScreenControlEvent::NextWindow)
                .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err.to_string()))?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::PreviousWindow) => {
            control_tx
                .send(ScreenControlEvent::PreviousWindow)
                .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err.to_string()))?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::KillWindow) => {
            control_tx
                .send(ScreenControlEvent::KillWindow)
                .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err.to_string()))?;
            write_response(stream, &ScreenIpcResponse::Accepted)
        }
        Ok(ScreenIpcRequest::Ping) => write_response(stream, &ScreenIpcResponse::Accepted),
        Ok(ScreenIpcRequest::Quit) => {
            control_tx
                .send(ScreenControlEvent::Terminate)
                .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err.to_string()))?;
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

fn stream_attach(
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

fn write_result_response(stream: &mut LocalSocketStream, result: io::Result<()>) -> io::Result<()> {
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

fn write_response(stream: &mut LocalSocketStream, response: &ScreenIpcResponse) -> io::Result<()> {
    serde_json::to_writer(&mut *stream, response)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    stream.write_all(b"\n")?;
    stream.flush()
}
