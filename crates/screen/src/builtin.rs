use std::{
    error::Error,
    io::{self, Read, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};

use crossterm::{
    event::{self, Event},
    terminal::{self, size as terminal_size},
};

use crate::{
    ScreenArgs,
    ipc::ScreenIpcEndpoint,
    pty_process::spawn_screen_pty,
    service::ScreenSessionService,
    session_core::{ScreenControlEvent, ScreenSessionBus},
    sessions::register_builtin_screen_session,
    terminal_input::key_to_bytes,
};

struct RawMode;

impl RawMode {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

fn screen_session_endpoint(args: &ScreenArgs) -> ScreenIpcEndpoint {
    args.session_name
        .as_deref()
        .map(ScreenIpcEndpoint::for_new_session)
        .unwrap_or_else(|| ScreenIpcEndpoint::for_session("anonymous"))
}

pub(crate) fn run_builtin_screen(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    let endpoint = screen_session_endpoint(&args);
    let session_name_state = args
        .session_name
        .as_ref()
        .map(|name| Arc::new(Mutex::new(name.clone())));
    let _session_record =
        register_builtin_screen_session(&args, &endpoint, session_name_state.clone())?;
    let session_bus = ScreenSessionBus::new();
    let (control_tx, control_rx) = mpsc::channel::<ScreenControlEvent>();
    let _session_service = ScreenSessionService::start(
        session_name_state,
        endpoint,
        session_bus.clone(),
        control_tx,
    )?;
    let _raw = RawMode::enter()?;
    let (cols, rows) = resolve_size(args.cols, args.rows);

    let mut pty = spawn_screen_pty(&args, cols, rows)?;
    let mut reader = pty.take_reader()?;

    let should_run = Arc::new(AtomicBool::new(true));
    let mut stdout = io::stdout();

    let output_running = Arc::clone(&should_run);
    let output_bus = session_bus.clone();
    let output_thread = thread::spawn(move || {
        let mut buf = [0u8; 8192];
        while output_running.load(Ordering::Acquire) {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    output_bus.publish_output(&buf[..n]);
                    if stdout.write_all(&buf[..n]).is_err() {
                        break;
                    }
                    if stdout.flush().is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let mut exit_code: Option<i32> = None;

    loop {
        match pty.try_wait_code() {
            Ok(Some(code)) => {
                session_bus.publish_exit(code);
                exit_code = Some(code);
                break;
            }
            Ok(None) => {}
            Err(_) => {
                exit_code = Some(-1);
                break;
            }
        }

        let mut terminate_requested = false;
        while let Ok(control) = control_rx.try_recv() {
            match control {
                ScreenControlEvent::Input(bytes) => {
                    if pty.write_input(&bytes).is_err() {
                        break;
                    }
                }
                ScreenControlEvent::Resize { cols, rows } => {
                    pty.resize(cols, rows);
                    session_bus.publish_resize(cols, rows);
                }
                ScreenControlEvent::Terminate => {
                    let _ = pty.kill();
                    terminate_requested = true;
                    break;
                }
            }
        }
        if terminate_requested {
            thread::sleep(Duration::from_millis(16));
            continue;
        }

        match event::poll(Duration::from_millis(16)) {
            Ok(true) => match event::read() {
                Ok(Event::Key(key)) => {
                    if let Some(bytes) = key_to_bytes(key) {
                        if pty.write_input(&bytes).is_err() {
                            break;
                        }
                    }
                }
                Ok(Event::Resize(cols, rows)) => {
                    pty.resize(cols, rows);
                    session_bus.publish_resize(cols, rows);
                }
                Ok(_) => {}
                Err(_) => break,
            },
            Ok(false) => {}
            Err(_) => break,
        }
    }

    if exit_code.is_none() {
        let _ = pty.kill();
        if let Ok(code) = pty.wait_code() {
            exit_code = Some(code);
        }
    }
    should_run.store(false, Ordering::Release);
    let _ = output_thread.join();

    let exit_code = exit_code.unwrap_or(-1);
    if exit_code == 0 {
        Ok(())
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            terman_common::builtin_screen_failure_hint(exit_code),
        )))
    }
}

fn resolve_size(cols_override: Option<u16>, rows_override: Option<u16>) -> (u16, u16) {
    let (cols, rows) = terminal_size().unwrap_or((120, 32));
    (cols_override.unwrap_or(cols), rows_override.unwrap_or(rows))
}