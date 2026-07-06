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

use super::{
    attach_output::{
        print_attach_displays, print_attach_hardcopy, print_attach_help, print_attach_info,
        print_attach_license, print_attach_time, print_attach_version, print_attach_windows,
    },
    attach_size::{fit_attach_window, toggle_attach_width},
    attach_termcap::print_attach_dumptermcap,
    ipc_client::send_control_request,
};
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
    client_id: String,
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
                    Some(ScreenInputAction::Clear) => {
                        send_control_request(&endpoint, ScreenIpcRequest::Clear)?;
                    }
                    Some(ScreenInputAction::Reset) => {
                        send_control_request(&endpoint, ScreenIpcRequest::Reset)?;
                    }
                    Some(ScreenInputAction::Resize) => {
                        sync_attach_terminal_size(&endpoint)?;
                    }
                    Some(ScreenInputAction::Detach) => {
                        send_control_request(
                            &endpoint,
                            ScreenIpcRequest::DetachClient {
                                client_id: client_id.clone(),
                            },
                        )?;
                        running.store(false, Ordering::Release);
                        return Ok(());
                    }
                    Some(ScreenInputAction::DetachAll) => {
                        send_control_request(&endpoint, ScreenIpcRequest::DetachAll)?;
                        running.store(false, Ordering::Release);
                        return Ok(());
                    }
                    Some(ScreenInputAction::Displays) => print_attach_displays(&endpoint)?,
                    Some(ScreenInputAction::Fit) => fit_attach_window(&endpoint)?,
                    Some(ScreenInputAction::DumpTermcap) => print_attach_dumptermcap(&endpoint)?,
                    Some(ScreenInputAction::Help) => print_attach_help()?,
                    Some(ScreenInputAction::Hardcopy) => print_attach_hardcopy(&endpoint)?,
                    Some(ScreenInputAction::Info) => print_attach_info(&endpoint)?,
                    Some(ScreenInputAction::NewWindow) => {
                        send_control_request(
                            &endpoint,
                            ScreenIpcRequest::NewWindow { command: None },
                        )?;
                    }
                    Some(ScreenInputAction::Paste) => {
                        send_control_request(&endpoint, ScreenIpcRequest::PasteBuffer)?;
                    }
                    Some(ScreenInputAction::Time) => print_attach_time()?,
                    Some(ScreenInputAction::Version) => print_attach_version()?,
                    Some(ScreenInputAction::Windows) => print_attach_windows(&endpoint)?,
                    Some(ScreenInputAction::WidthToggle) => toggle_attach_width(&endpoint)?,
                    Some(ScreenInputAction::LastWindow) => {
                        send_control_request(&endpoint, ScreenIpcRequest::LastWindow)?;
                    }
                    Some(ScreenInputAction::NextWindow) => {
                        send_control_request(&endpoint, ScreenIpcRequest::NextWindow)?;
                    }
                    Some(ScreenInputAction::SelectWindow(index)) => {
                        send_control_request(
                            &endpoint,
                            ScreenIpcRequest::SelectWindow { index },
                        )?;
                    }
                    Some(ScreenInputAction::PreviousWindow) => {
                        send_control_request(&endpoint, ScreenIpcRequest::PreviousWindow)?;
                    },
                    Some(ScreenInputAction::License) => print_attach_license()?,
                    Some(ScreenInputAction::Kill) => {
                        send_control_request(&endpoint, ScreenIpcRequest::KillWindow)?;
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
            terman_common::builtin_screen_attach_output_thread_panicked_hint(),
        )),
    }
}

fn sync_attach_terminal_size(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;
    send_control_request(endpoint, ScreenIpcRequest::Resize { cols, rows })
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
            ScreenIpcResponse::PasteBuffer { .. } => {}
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