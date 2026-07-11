use std::{
    io::{self, BufRead, BufReader},
    sync::{Arc, Mutex, mpsc},
};

use interprocess::local_socket::prelude::*;

use crate::{
    ipc::{TmuxIpcRequest, TmuxIpcResponse},
    pane_service::{capture_pane, clear_history, kill_pane, resize_pane, select_pane,
        split_pane, swap_pane, toggle_pane_zoom, write_pane_info},
    service_codec::write_response,
    service_buffer::{delete_buffer, get_buffer, list_buffers, paste_buffer, set_buffer},
    session_core::{TmuxControlEvent, TmuxSessionBus, TmuxSessionEvent},
    window_option_service::set_synchronize_panes,
};

pub(crate) fn handle_client(
    stream: &mut LocalSocketStream,
    session_name: &Arc<Mutex<String>>,
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
        Ok(TmuxIpcRequest::CapturePane { window, pane }) => capture_pane(stream, bus, window, pane),
        Ok(TmuxIpcRequest::DeleteBuffer { name }) => delete_buffer(stream, bus, name),
        Ok(TmuxIpcRequest::GetBuffer { name }) => get_buffer(stream, bus, name),
        Ok(TmuxIpcRequest::ListBuffers) => list_buffers(stream, bus),
        Ok(TmuxIpcRequest::PasteBuffer { name }) => {
            paste_buffer(stream, bus, control_tx, name)
        }
        Ok(TmuxIpcRequest::RefreshClient) => write_response(
            stream,
            &TmuxIpcResponse::Captured {
                bytes: bus.replay_snapshot(),
            },
        ),
        Ok(TmuxIpcRequest::SetBuffer { name, bytes }) => {
            set_buffer(stream, bus, name, bytes)
        }
        Ok(TmuxIpcRequest::DetachAll) => {
            bus.publish_detach();
            write_response(stream, &TmuxIpcResponse::Accepted)
        }
        Ok(TmuxIpcRequest::ClearHistory { window, pane }) => clear_history(stream, bus, window, pane),
        Ok(TmuxIpcRequest::DisplayMessage { message }) => {
            let mut bytes = message.into_bytes();
            bytes.extend_from_slice(b"\r\n");
            bus.publish_transient_output(&bytes);
            write_response(stream, &TmuxIpcResponse::Accepted)
        }
        Ok(TmuxIpcRequest::Info) => write_info(stream, session_name, cwd, bus),
        Ok(TmuxIpcRequest::Input { bytes }) => accept_control(stream, control_tx, TmuxControlEvent::Input(bytes)),
        Ok(TmuxIpcRequest::Ping) => write_response(stream, &TmuxIpcResponse::Accepted),
        Ok(TmuxIpcRequest::Quit) => accept_control(stream, control_tx, TmuxControlEvent::Terminate),
        Ok(TmuxIpcRequest::RenameWindow { index, name }) => {
            accept_control(stream, control_tx, TmuxControlEvent::RenameWindow { index, name })
        }
        Ok(TmuxIpcRequest::RenameSession { name }) => {
            rename_session(session_name, name)?;
            write_response(stream, &TmuxIpcResponse::Accepted)
        }
        Ok(TmuxIpcRequest::KillWindow { index }) => {
            accept_control(stream, control_tx, TmuxControlEvent::KillWindow { index })
        }
        Ok(TmuxIpcRequest::NewWindow { index, name, command }) => {
            accept_control(stream, control_tx, TmuxControlEvent::NewWindow { index, name, command })
        }
        Ok(TmuxIpcRequest::UpdateWindows { windows }) => {
            bus.set_windows(windows);
            write_response(stream, &TmuxIpcResponse::Accepted)
        }
        Ok(TmuxIpcRequest::SelectWindow { index }) => select_window(stream, bus, control_tx, index),
        Ok(TmuxIpcRequest::LastWindow) => select_last_window(stream, bus, control_tx),
        Ok(TmuxIpcRequest::PaneInfo { window }) => write_pane_info(stream, bus, window),
        Ok(TmuxIpcRequest::SplitPane { window, horizontal, command }) => split_pane(stream, bus, control_tx, window, horizontal, command),
        Ok(TmuxIpcRequest::SelectPane { window, pane }) => select_pane(stream, bus, control_tx, window, pane),
        Ok(TmuxIpcRequest::SwapPane { window, source, target, forward }) => {
            swap_pane(stream, bus, control_tx, window, source, target, forward)
        }
        Ok(TmuxIpcRequest::KillPane { window, pane }) => kill_pane(stream, bus, control_tx, window, pane),
        Ok(TmuxIpcRequest::TogglePaneZoom { window, pane }) => toggle_pane_zoom(stream, bus, control_tx, window, pane),
        Ok(TmuxIpcRequest::SetSynchronizePanes { window, enabled }) =>
            set_synchronize_panes(stream, bus, control_tx, window, enabled),
        Ok(TmuxIpcRequest::ResizePane { window, pane, cols, rows }) => resize_pane(stream, bus, control_tx, (window, pane), (cols, rows)),
        Ok(TmuxIpcRequest::Resize { cols, rows }) => {
            accept_control(stream, control_tx, TmuxControlEvent::Resize { cols, rows })
        }
        Err(err) => write_response(stream, &TmuxIpcResponse::Rejected { reason: err.to_string() }),
    }
}


fn write_info(
    stream: &mut LocalSocketStream,
    session_name: &Arc<Mutex<String>>,
    cwd: &str,
    bus: &TmuxSessionBus,
) -> io::Result<()> {
    let status = bus.status_snapshot();
    write_response(
        stream,
        &TmuxIpcResponse::Info {
            session_name: current_session_name(session_name)?,
            windows: status.windows,
            attached_clients: status.attached_clients,
            active_window: status.active_window,
            window_indexes: status.window_indexes,
            window_names: status.window_names,
            cwd: cwd.to_string(),
        },
    )
}

fn select_window(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    control_tx: &mpsc::Sender<TmuxControlEvent>,
    index: u32,
) -> io::Result<()> {
    if bus.select_window(index) {
        accept_control(stream, control_tx, TmuxControlEvent::SelectWindow { index })
    } else {
        write_window_missing(stream, "current", index as usize)
    }
}

fn select_last_window(
    stream: &mut LocalSocketStream,
    bus: &TmuxSessionBus,
    control_tx: &mpsc::Sender<TmuxControlEvent>,
) -> io::Result<()> {
    if let Some(index) = bus.select_last_window() {
        accept_control(stream, control_tx, TmuxControlEvent::SelectWindow { index })
    } else {
        write_window_missing(stream, "last", 0)
    }
}

fn accept_control(
    stream: &mut LocalSocketStream,
    control_tx: &mpsc::Sender<TmuxControlEvent>,
    event: TmuxControlEvent,
) -> io::Result<()> {
    send_control(control_tx, event)?;
    write_response(stream, &TmuxIpcResponse::Accepted)
}

fn current_session_name(session_name: &Arc<Mutex<String>>) -> io::Result<String> {
    session_name
        .lock()
        .map(|name| name.clone())
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))
}

fn rename_session(session_name: &Arc<Mutex<String>>, name: String) -> io::Result<()> {
    let mut session_name = session_name
        .lock()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;
    *session_name = name;
    Ok(())
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
        let should_close = matches!(response, TmuxIpcResponse::Detached | TmuxIpcResponse::Exit { .. });
        write_response(stream, &response)?;
        if should_close { break; }
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

fn write_window_missing(
    stream: &mut LocalSocketStream,
    target: &str,
    index: usize,
) -> io::Result<()> {
    write_response(
        stream,
        &TmuxIpcResponse::Rejected {
            reason: terman_common::builtin_tmux_window_not_found_hint(target, index),
        },
    )
}