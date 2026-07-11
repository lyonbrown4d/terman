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
    attach_actions::{AttachActionResult, handle_attach_action, sync_attach_terminal_size},
    attach_command::prompt_attach_command,
    attach_title::prompt_attach_title,
    attach_copy::{finish_attach_copy_mode, start_attach_copy_mode},
    attach_mouse::{AttachMouseState, disable_mouse_capture, enable_mouse_capture, handle_attach_mouse, handle_attach_window_list_key, open_attach_window_list},
    attach_select::prompt_attach_select,
    ipc_client::send_control_request,
};
use crate::{
    copy_mode::{ScreenCopyMode, ScreenCopyResult},
    ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse},
    terminal_input::ScreenInputDecoder,
};

struct AttachRawMode;

impl AttachRawMode {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        enable_mouse_capture()?;
        Ok(Self)
    }
}

impl Drop for AttachRawMode {
    fn drop(&mut self) {
        disable_mouse_capture();
        let _ = terminal::disable_raw_mode();
    }
}

pub(super) fn attach_interactive(
    endpoint: ScreenIpcEndpoint,
    stream: LocalSocketStream,
    client_id: String,
    session_name: String,
) -> io::Result<()> {
    let _raw = AttachRawMode::enter()?;
    sync_attach_terminal_size(&endpoint)?;

    let running = Arc::new(AtomicBool::new(true));
    let output_paused = Arc::new(AtomicBool::new(false));
    let output_running = Arc::clone(&running);
    let output_output_paused = Arc::clone(&output_paused);
    let output_thread = thread::spawn(move || {
        let result = read_attach_stream(stream, &output_output_paused);
        output_running.store(false, Ordering::Release);
        result
    });

    let mut input_decoder = ScreenInputDecoder::new();
    let mut mouse_state = AttachMouseState::default();
    let mut copy_mode: Option<ScreenCopyMode> = None;
    while running.load(Ordering::Acquire) {
        match event::poll(Duration::from_millis(16)) {
            Ok(true) => match event::read() {
                Ok(Event::Mouse(mouse)) => {
                    if let Some(mode) = copy_mode.as_mut() {
                        if mode.handle_mouse(mouse) {
                            mode.render()?;
                        }
                    } else {
                        handle_attach_mouse(&endpoint, &mut mouse_state, mouse)?;
                        output_paused.store(mouse_state.list_open(), Ordering::Release);
                    }
                }
                Ok(Event::Key(key)) => {
                    if let Some(mode) = copy_mode.as_mut() {
                        match mode.handle_key(key) {
                            ScreenCopyResult::Continue => mode.render()?,
                            ScreenCopyResult::Cancel => {
                                copy_mode = None;
                                output_paused.store(false, Ordering::Release);
                                finish_attach_copy_mode(&endpoint, None)?;
                            }
                            ScreenCopyResult::Copy(bytes) => {
                                copy_mode = None;
                                output_paused.store(false, Ordering::Release);
                                finish_attach_copy_mode(&endpoint, Some(bytes))?;
                            }
                        }
                        continue;
                    }
                    if handle_attach_window_list_key(&endpoint, &mut mouse_state, &key)? {
                        output_paused.store(mouse_state.list_open(), Ordering::Release);
                        continue;
                    }
                    if let Some(action) = input_decoder.decode_key(key) {
                        match handle_attach_action(&endpoint, &client_id, action)? {
                            AttachActionResult::Continue => {}
                            AttachActionResult::CopyMode => {
                                let mode = start_attach_copy_mode(&endpoint)?;
                                output_paused.store(true, Ordering::Release);
                                mode.render()?;
                                copy_mode = Some(mode);
                            }
                            AttachActionResult::WindowList => {
                                output_paused.store(true, Ordering::Release);
                                open_attach_window_list(&endpoint, &mut mouse_state)?;
                            }
                            AttachActionResult::CommandPrompt => {
                                run_attach_prompt(&endpoint, &output_paused, || {
                                    prompt_attach_command(&session_name)
                                })?;
                            }
                            AttachActionResult::TitlePrompt => {
                                run_attach_prompt(&endpoint, &output_paused, || {
                                    prompt_attach_title(&endpoint)
                                })?;
                            }
                            AttachActionResult::SelectPrompt => {
                                run_attach_prompt(&endpoint, &output_paused, || {
                                    prompt_attach_select(&endpoint)
                                })?;
                            }
                            AttachActionResult::Stop => {
                                running.store(false, Ordering::Release);
                                return Ok(());
                            }
                        }
                    }
                }
                Ok(Event::Resize(cols, rows)) => {
                    send_control_request(&endpoint, ScreenIpcRequest::Resize { cols, rows })?;
                    if let Some(mode) = copy_mode.as_mut() {
                        mode.resize(cols, rows);
                        mode.render()?;
                    } else if mouse_state.list_open() {
                        open_attach_window_list(&endpoint, &mut mouse_state)?;
                    }
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

fn run_attach_prompt(
    endpoint: &ScreenIpcEndpoint,
    output_paused: &AtomicBool,
    prompt: impl FnOnce() -> io::Result<()>,
) -> io::Result<()> {
    output_paused.store(true, Ordering::Release);
    let result = prompt();
    output_paused.store(false, Ordering::Release);
    result?;
    let _ = send_control_request(endpoint, ScreenIpcRequest::Redisplay);
    Ok(())
}
fn read_attach_stream(
    stream: LocalSocketStream,
    output_paused: &AtomicBool,
) -> io::Result<()> {
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
                if !output_paused.load(Ordering::Acquire) {
                    stdout.write_all(&bytes)?;
                    stdout.flush()?;
                }
            }
            ScreenIpcResponse::Resize { .. } => {}
            ScreenIpcResponse::Exit { .. } => return Ok(()),
            ScreenIpcResponse::Rejected { reason } => {
                return Err(io::Error::new(io::ErrorKind::Unsupported, reason));
            }
        }
    }
}