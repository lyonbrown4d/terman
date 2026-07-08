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
    builtin_mouse::ScreenMouseState,
    builtin_output::{drain_window_output, handle_window_exit, publish_error, publish_window_redraw},
    builtin_runtime::{RawMode, poll_terminal_event, resolve_size, screen_session_endpoint},
    service::ScreenSessionService,
    session_core::{ScreenControlEvent, ScreenSessionBus},
    sessions::register_builtin_screen_session,
    window_runtime::{
        ScreenWindowOutput, ScreenWindowSwitch, apply_default_window_log, kill_active_window,
        kill_windows, new_screen_window_title, next_screen_window_index, renumber_screen_window,
        resize_windows, spawn_screen_window_runtime, switch_screen_window, write_active_window_input,
    },
};

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
    let mut mouse_state = ScreenMouseState::default();
    let mut exit_code: Option<i32> = None;

    loop {
        drain_window_output(&session_bus, &output_rx, active_window);
        if let Some(code) = handle_window_exit(&session_bus, &mut windows, &mut active_window) {
            session_bus.publish_exit(code);
            exit_code = Some(code);
            break;
        }

        let mut terminate_requested = false;
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
                            if let Err(err) = apply_default_window_log(&session_bus, &default_env) { publish_error(&session_bus, err); }
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
                    kill_windows(&mut windows);
                    terminate_requested = true;
                    break;
                }
            }
        }
        if terminate_requested {
            thread::sleep(Duration::from_millis(16));
            continue;
        }

        if poll_terminal_event(&session_bus, &mut windows, &mut active_window, &mut mouse_state).is_err() {
            break;
        }
    }

    if exit_code.is_none() {
        kill_windows(&mut windows);
        if let Some(window) = windows.first_mut() {
            if let Ok(code) = window.pty.wait_code() {
                exit_code = Some(code);
            }
        }
    }
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