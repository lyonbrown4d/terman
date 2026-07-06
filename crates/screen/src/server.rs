use std::{
    error::Error,
    io::{self, Read},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

use crate::{
    ScreenArgs,
    ipc::ScreenIpcEndpoint,
    pty_process::spawn_screen_pty,
    service::ScreenSessionService,
    session_core::{ScreenControlEvent, ScreenSessionBus},
    sessions::register_builtin_screen_session,
};

pub(crate) fn run_screen_server(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    let Some(session_name) = args.session_name.as_deref() else {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_internal_server_session_required_hint(),
        )));
    };
    let session_name_state = Arc::new(Mutex::new(session_name.to_string()));
    let endpoint = args
        .internal_endpoint_name
        .as_deref()
        .map(ScreenIpcEndpoint::from_raw_name)
        .unwrap_or_else(|| ScreenIpcEndpoint::for_session(session_name));
    let _session_record =
        register_builtin_screen_session(&args, &endpoint, Some(session_name_state.clone()))?;
    let session_bus = ScreenSessionBus::new();
    let (control_tx, control_rx) = mpsc::channel::<ScreenControlEvent>();
    let _session_service = ScreenSessionService::start(
        Some(session_name_state),
        endpoint,
        session_bus.clone(),
        control_tx,
    )?;

    let cols = args.cols.unwrap_or(120);
    let rows = args.rows.unwrap_or(32);
    session_bus.publish_resize(cols, rows);

    let mut pty = spawn_screen_pty(&args, cols, rows)?;
    let mut reader = pty.take_reader()?;

    let output_bus = session_bus.clone();
    let output_thread = thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => output_bus.publish_output(&buf[..n]),
                Err(_) => break,
            }
        }
    });

    let mut terminate_requested = false;
    let exit_code = loop {
        match pty.try_wait_code() {
            Ok(Some(code)) => {
                session_bus.publish_exit(code);
                break code;
            }
            Ok(None) => {}
            Err(_) => {
                session_bus.publish_exit(-1);
                break -1;
            }
        }

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
                    if !terminate_requested {
                        terminate_requested = true;
                        let _ = pty.kill();
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(16));
    };

    let _ = output_thread.join();

    if exit_code == 0 {
        Ok(())
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            terman_common::builtin_screen_internal_server_exited_hint(exit_code),
        )))
    }
}