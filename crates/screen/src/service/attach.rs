use std::{
    io::{self, BufRead, BufReader, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use crossterm::{
    event::{self, Event},
    terminal,
};
use interprocess::local_socket::prelude::*;

use super::ipc_client::send_control_request;
use crate::{
    ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse},
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

pub(super) fn attach_interactive(
    endpoint: ScreenIpcEndpoint,
    stream: LocalSocketStream,
) -> io::Result<()> {
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
                    Some(ScreenInputAction::Help) => print_attach_help()?,
                    Some(ScreenInputAction::Kill) => {
                        send_control_request(&endpoint, ScreenIpcRequest::Quit)?;
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

fn print_attach_help() -> io::Result<()> {
    let mut stdout = io::stdout();
    stdout.write_all(b"\r\n")?;
    stdout.write_all(terman_common::builtin_screen_attach_help_hint().as_bytes())?;
    stdout.write_all(b"\r\n")?;
    stdout.flush()
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
            ScreenIpcResponse::Detached => return Ok(()),
            ScreenIpcResponse::Hardcopy { .. } => {}
            ScreenIpcResponse::Info { .. } => {}
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
