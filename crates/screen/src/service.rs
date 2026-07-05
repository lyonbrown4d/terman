use std::{
    io::{self, BufRead, BufReader, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use crossterm::{
    event::{self, Event},
    terminal,
};
use interprocess::local_socket::prelude::*;

use crate::{
    ScreenArgs,
    ipc::{ScreenAttachMode, ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse},
    session_core::{ScreenControlEvent, ScreenSessionBus, ScreenSessionEvent},
    sessions::find_builtin_screen_session_for_attach,
    terminal_input::{ScreenInputAction, ScreenInputDecoder},
};

struct AttachRawMode;

impl AttachRawMode {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for AttachRawMode {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

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

    attach_interactive(endpoint, stream)
}

pub(crate) fn request_screen_control_command(args: &ScreenArgs) -> io::Result<()> {
    let Some(command_text) = args
        .execute
        .as_deref()
        .map(str::trim)
        .filter(|command| !command.is_empty())
    else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_command_required_hint(),
        ));
    };

    let (command, inline_payload) = split_control_command(command_text);
    match command.to_ascii_lowercase().as_str() {
        "quit" => send_session_control_request(args, ScreenIpcRequest::Quit),
        "stuff" => {
            let payload = control_command_payload(inline_payload, &args.execute_args);
            if payload.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    terman_common::builtin_screen_control_stuff_required_hint(),
                ));
            }
            send_session_control_request(
                args,
                ScreenIpcRequest::Input {
                    bytes: decode_stuff_payload(&payload),
                },
            )
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_command_unsupported_hint(command),
        )),
    }
}

fn split_control_command(command: &str) -> (&str, &str) {
    let command = command.trim();
    let command_end = command.find(char::is_whitespace).unwrap_or(command.len());
    let verb = &command[..command_end];
    let payload = command[command_end..].trim_start();
    (verb, payload)
}

fn control_command_payload(inline_payload: &str, args: &[String]) -> String {
    let mut payload = String::new();
    if !inline_payload.is_empty() {
        payload.push_str(inline_payload);
    }
    for arg in args {
        if !payload.is_empty() {
            payload.push(' ');
        }
        payload.push_str(arg);
    }
    payload
}

fn decode_stuff_payload(payload: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(payload.len());
    let mut chars = payload.chars();

    while let Some(ch) = chars.next() {
        if ch != '\\' {
            push_utf8(&mut bytes, ch);
            continue;
        }

        match chars.next() {
            Some('n') => bytes.push(b'\n'),
            Some('r') => bytes.push(b'\r'),
            Some('t') => bytes.push(b'\t'),
            Some('\\') => bytes.push(b'\\'),
            Some(other) => {
                bytes.push(b'\\');
                push_utf8(&mut bytes, other);
            }
            None => bytes.push(b'\\'),
        }
    }

    bytes
}

fn push_utf8(bytes: &mut Vec<u8>, ch: char) {
    let mut buf = [0u8; 4];
    bytes.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
}

fn send_session_control_request(args: &ScreenArgs, request: ScreenIpcRequest) -> io::Result<()> {
    let session = find_builtin_screen_session_for_attach(args.session_name.as_deref())?;
    let endpoint = session
        .ipc_endpoint
        .as_deref()
        .map(ScreenIpcEndpoint::from_raw_name)
        .unwrap_or_else(|| ScreenIpcEndpoint::for_session(&session.name));

    send_control_request(&endpoint, request)
}

fn attach_interactive(endpoint: ScreenIpcEndpoint, stream: LocalSocketStream) -> io::Result<()> {
    let _raw = AttachRawMode::enter()?;
    sync_attach_terminal_size(&endpoint)?;

    let running = Arc::new(AtomicBool::new(true));
    let output_running = Arc::clone(&running);
    let output_thread = thread::spawn(move || {
        let result = read_attach_stream(stream);
        output_running.store(false, Ordering::Release);
        result
    });

    let mut input_decoder = ScreenInputDecoder::new();
    while running.load(Ordering::Acquire) {
        match event::poll(Duration::from_millis(16)) {
            Ok(true) => match event::read() {
                Ok(Event::Key(key)) => match input_decoder.decode_key(key) {
                    Some(ScreenInputAction::Bytes(bytes)) => {
                        send_control_request(&endpoint, ScreenIpcRequest::Input { bytes })?;
                    }
                    Some(ScreenInputAction::Detach) => {
                        send_control_request(&endpoint, ScreenIpcRequest::Detach)?;
                        running.store(false, Ordering::Release);
                        return Ok(());
                    }
                    None => {}
                },
                Ok(Event::Resize(cols, rows)) => {
                    send_control_request(&endpoint, ScreenIpcRequest::Resize { cols, rows })?;
                }
                Ok(_) => {}
                Err(err) => return Err(err),
            },
            Ok(false) => {}
            Err(err) => return Err(err),
        }
    }

    match output_thread.join() {
        Ok(result) => result,
        Err(_) => Err(io::Error::new(
            io::ErrorKind::Other,
            "screen attach output thread panicked",
        )),
    }
}

fn sync_attach_terminal_size(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;
    send_control_request(endpoint, ScreenIpcRequest::Resize { cols, rows })
}

fn send_control_request(endpoint: &ScreenIpcEndpoint, request: ScreenIpcRequest) -> io::Result<()> {
    let mut stream = endpoint.connect_options()?.connect_sync()?;
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
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected screen control response",
        )),
    }
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
        Ok(ScreenIpcRequest::Attach { .. }) => stream_attach(stream, bus),
        Ok(ScreenIpcRequest::Detach) => write_response(stream, &ScreenIpcResponse::Accepted),
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