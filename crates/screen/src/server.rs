use std::{
    error::Error,
    io,
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

use crate::{
    ScreenArgs,
    ipc::ScreenIpcEndpoint,
    service::ScreenSessionService,
    session_core::{ScreenControlEvent, ScreenSessionBus},
    sessions::register_builtin_screen_session,
    window_runtime::{
        ScreenWindowOutput, ScreenWindowRuntime, kill_windows, resize_windows,
        spawn_screen_window_runtime, write_active_window_input,
    },
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

    let (output_tx, output_rx) = mpsc::channel::<ScreenWindowOutput>();
    let mut windows = vec![spawn_screen_window_runtime(
        &args,
        0,
        args.command.clone(),
        cols,
        rows,
        output_tx.clone(),
    )?];
    let mut active_window = 0;
    let mut terminate_requested = false;

    let exit_code = loop {
        drain_window_output(&session_bus, &output_rx);
        if let Some(code) = first_exited_window_code(&mut windows) {
            session_bus.publish_exit(code);
            break code;
        }

        while let Ok(control) = control_rx.try_recv() {
            match control {
                ScreenControlEvent::Input(bytes) => {
                    write_active_window_input(&mut windows, active_window, &bytes);
                }
                ScreenControlEvent::NewWindow { command } => {
                    let index = windows.len();
                    match spawn_screen_window_runtime(
                        &args,
                        index,
                        command.clone(),
                        cols,
                        rows,
                        output_tx.clone(),
                    ) {
                        Ok(window) => {
                            session_bus.add_window(index, command);
                            active_window = index;
                            windows.push(window);
                            session_bus.publish_transient_output(b"\x1bc");
                        }
                        Err(err) => publish_error(&session_bus, err),
                    }
                }
                ScreenControlEvent::Resize { cols, rows } => {
                    resize_windows(&windows, cols, rows);
                    session_bus.publish_resize(cols, rows);
                }
                ScreenControlEvent::Terminate => {
                    if !terminate_requested {
                        terminate_requested = true;
                        kill_windows(&mut windows);
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(16));
    };

    if exit_code == 0 {
        Ok(())
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            terman_common::builtin_screen_internal_server_exited_hint(exit_code),
        )))
    }
}

fn drain_window_output(bus: &ScreenSessionBus, rx: &mpsc::Receiver<ScreenWindowOutput>) {
    while let Ok(output) = rx.try_recv() {
        bus.publish_window_output(output.index, &output.bytes);
    }
}

fn first_exited_window_code(windows: &mut [ScreenWindowRuntime]) -> Option<i32> {
    windows
        .iter_mut()
        .find_map(|window| window.pty.try_wait_code().ok().flatten())
}

fn publish_error(bus: &ScreenSessionBus, err: Box<dyn Error>) {
    let message = format!("\r\nscreen window failed: {err}\r\n");
    bus.publish_transient_output(message.as_bytes());
}