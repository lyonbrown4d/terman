use crate::{blanker::ScreenBlanker, copy_mode::ScreenCopyMode};
use std::{error::Error, io, sync::{Arc, Mutex, mpsc}};

use crate::{
    ScreenArgs,
    builtin_control::{BuiltinControlDefaults, drain_builtin_controls},
    builtin_mouse::ScreenMouseState,
    builtin_output::{drain_window_output, handle_window_exit},
    builtin_runtime::{RawMode, poll_terminal_event, resolve_size, screen_session_endpoint},
    service::ScreenSessionService,
    session_core::{ScreenControlEvent, ScreenSessionBus},
    terminal_input::ScreenInputDecoder,
    sessions::register_builtin_screen_session,
    window_runtime::{ScreenWindowOutput, kill_windows, spawn_screen_window_runtime},
};

pub(crate) fn run_builtin_screen(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    let endpoint = screen_session_endpoint(&args);
    let runtime_endpoint = endpoint.clone();
    let session_name_state = args
        .session_name
        .as_ref()
        .map(|name| Arc::new(Mutex::new(name.clone())));
    let _session_record =
        register_builtin_screen_session(&args, &endpoint, session_name_state.clone())?;
    let session_bus = ScreenSessionBus::new();
    let (control_tx, control_rx) = mpsc::channel::<ScreenControlEvent>();
    let runtime_control_tx = control_tx.clone();
    let _session_service = ScreenSessionService::start(
        session_name_state,
        endpoint,
        session_bus.clone(),
        control_tx,
    )?;
    let _raw = RawMode::enter()?;
    let size = resolve_size(args.cols, args.rows);
    session_bus.publish_resize(size.0, size.1);

    let mut defaults = BuiltinControlDefaults::new(session_bus.status_snapshot().scrollback_lines);
    let (output_tx, output_rx) = mpsc::channel::<ScreenWindowOutput>();
    let mut windows = vec![spawn_screen_window_runtime(
        &args,
        0,
        args.command.clone(),
        defaults.cwd.as_deref(),
        &defaults.env,
        size.0,
        size.1,
        output_tx.clone(),
    )?];
    let mut active_window = 0;
    let mut mouse_state = ScreenMouseState::default();
    let mut input_decoder = ScreenInputDecoder::new();
    let mut exit_code: Option<i32> = None;

    let mut copy_mode: Option<ScreenCopyMode> = None;
    let mut blanker = ScreenBlanker::default();
    loop {
        let display_output = copy_mode.is_none() && !mouse_state.list_open() && !blanker.is_active();
        drain_window_output(
            &session_bus,
            &output_rx,
            active_window,
            display_output,
        );
        session_bus.poll_silence();
        if let Some(code) = handle_window_exit(
            &session_bus,
            &mut windows,
            &mut active_window,
            display_output,
        ) {
            session_bus.publish_exit(code);
            exit_code = Some(code);
            break;
        }

        if drain_builtin_controls(
            &args,
            &session_bus,
            &control_rx,
            &output_tx,
            &mut windows,
            &mut active_window,
            &mut defaults,
            size,
            display_output,
        ) {
            continue;
        }

        if poll_terminal_event(
            &session_bus,
            &runtime_control_tx,
            &runtime_endpoint,
            &mut input_decoder,
            &mut copy_mode,
            &mut blanker,
            &mut windows,
            &mut active_window,
            &mut mouse_state,
        ).is_err() {
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