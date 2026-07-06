use std::{
    collections::BTreeMap,
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
        ScreenWindowOutput, ScreenWindowRuntime, ScreenWindowSwitch, kill_active_window, kill_windows, new_screen_window_title, next_screen_window_index, renumber_screen_window, resize_windows,
        spawn_screen_window_runtime, switch_screen_window, take_exited_window, write_active_window_input,
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

    let mut default_cwd = std::env::current_dir().ok();
    let mut default_env = BTreeMap::<String, Option<String>>::new();
    let mut default_scrollback_lines = session_bus.status_snapshot().scrollback_lines;
    let (output_tx, output_rx) = mpsc::channel::<ScreenWindowOutput>();
    let mut windows = vec![spawn_screen_window_runtime(
        &args,
        0,
        args.command.clone(),
        default_cwd.as_deref(),
        &default_env,
        cols,
        rows,
        output_tx.clone(),
    )?];
    let mut active_window = 0;
    let mut terminate_requested = false;

    let exit_code = loop {
        drain_window_output(&session_bus, &output_rx);
        if let Some(code) = handle_window_exit(
            &session_bus,
            &mut windows,
            &mut active_window,
        ) {
            session_bus.publish_exit(code);
            break code;
        }

        while let Ok(control) = control_rx.try_recv() {
            match control {
                ScreenControlEvent::Input(bytes) => {
                    write_active_window_input(&mut windows, active_window, &bytes);
                }
                ScreenControlEvent::SetDefaultCwd { path } => {
                    default_cwd = Some(path);
                }
                ScreenControlEvent::SetEnv { name, value } => {
                    default_env.insert(name, Some(value));
                }
                ScreenControlEvent::UnsetEnv { name } => {
                    default_env.insert(name, None);
                }
                ScreenControlEvent::SetDefaultScrollback { lines } => {
                    default_scrollback_lines = lines;
                }
                ScreenControlEvent::NewWindow { command } => {
                    let index = next_screen_window_index(&windows);
                    let title = new_screen_window_title(command.as_deref(), &default_env);
                    match spawn_screen_window_runtime(
                        &args,
                        index,
                        command.clone(),
                        default_cwd.as_deref(),
                        &default_env,
                        cols,
                        rows,
                        output_tx.clone(),
                    ) {
                        Ok(window) => {
                            session_bus.add_window_with_scrollback(index, title, default_scrollback_lines);
                            windows.push(window);
                            if let Some(replay) = switch_screen_window(
                                &session_bus,
                                &windows,
                                &mut active_window,
                                ScreenWindowSwitch::Select(index),
                            ) {
                                publish_window_redraw(&session_bus, &replay);
                            }
                        }
                        Err(err) => publish_error(&session_bus, err),
                    }
                }
                ScreenControlEvent::SelectWindow { index } => {
                    if let Some(replay) = switch_screen_window(
                        &session_bus,
                        &windows,
                        &mut active_window,
                        ScreenWindowSwitch::Select(index),
                    ) {
                        publish_window_redraw(&session_bus, &replay);
                    }
                }
                ScreenControlEvent::NextWindow => {
                    if let Some(replay) = switch_screen_window(
                        &session_bus,
                        &windows,
                        &mut active_window,
                        ScreenWindowSwitch::Next,
                    ) {
                        publish_window_redraw(&session_bus, &replay);
                    }
                }
                ScreenControlEvent::PreviousWindow => {
                    if let Some(replay) = switch_screen_window(
                        &session_bus,
                        &windows,
                        &mut active_window,
                        ScreenWindowSwitch::Previous,
                    ) {
                        publish_window_redraw(&session_bus, &replay);
                    }
                }
                ScreenControlEvent::LastWindow => {
                    if let Some(replay) = switch_screen_window(
                        &session_bus,
                        &windows,
                        &mut active_window,
                        ScreenWindowSwitch::Last,
                    ) {
                        publish_window_redraw(&session_bus, &replay);
                    }
                }
                ScreenControlEvent::KillWindow => {
                    kill_active_window(&mut windows, active_window);
                }
                ScreenControlEvent::NumberWindow { source, index } => {
                    if renumber_screen_window(&mut windows, source, index, &mut active_window) {
                        session_bus.renumber_window(source, index);
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

fn publish_window_redraw(bus: &ScreenSessionBus, replay: &[u8]) {
    bus.publish_transient_output(b"\x1bc");
    if !replay.is_empty() {
        bus.publish_transient_output(replay);
    }
}
fn drain_window_output(bus: &ScreenSessionBus, rx: &mpsc::Receiver<ScreenWindowOutput>) {
    while let Ok(output) = rx.try_recv() {
        bus.publish_window_output(output.index, &output.bytes);
    }
}

fn handle_window_exit(
    bus: &ScreenSessionBus,
    windows: &mut Vec<ScreenWindowRuntime>,
    active_window: &mut usize,
) -> Option<i32> {
    let exit = take_exited_window(windows)?;
    let removal = bus.remove_window(exit.index)?;
    if removal.last_window {
        return Some(exit.code);
    }
    if let Some(index) = removal.active_window {
        *active_window = index;
    }
    if removal.redraw {
        publish_window_redraw(bus, &removal.replay);
    }
    None
}

fn publish_error(bus: &ScreenSessionBus, err: Box<dyn Error>) {
    let message = format!("\r\nscreen window failed: {err}\r\n");
    bus.publish_transient_output(message.as_bytes());
}