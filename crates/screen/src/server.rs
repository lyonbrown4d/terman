use std::{error::Error, io, sync::{Arc, Mutex, mpsc}, thread, time::Duration};

use crate::{
    ScreenArgs,
    ipc::ScreenIpcEndpoint,
    server_control::{
        ServerControlDefaults, drain_server_controls, drain_server_window_output,
        handle_server_window_exit,
    },
    server_manifest::sync_session_manifest,
    service::ScreenSessionService,
    session_core::{ScreenControlEvent, ScreenSessionBus},
    sessions::register_builtin_screen_session,
    window_runtime::{ScreenWindowOutput, spawn_screen_window_runtime},
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
    let endpoint_name = endpoint.raw_name().to_string();
    let _session_record = register_builtin_screen_session(&args, &endpoint, Some(session_name_state.clone()))?;
    let session_bus = ScreenSessionBus::new();
    let (control_tx, control_rx) = mpsc::channel::<ScreenControlEvent>();
    let _session_service = ScreenSessionService::start(
        Some(session_name_state.clone()),
        endpoint,
        session_bus.clone(),
        control_tx,
    )?;

    let size = (args.cols.unwrap_or(120), args.rows.unwrap_or(32));
    session_bus.publish_resize(size.0, size.1);

    let mut defaults = ServerControlDefaults::new(session_bus.status_snapshot().scrollback_lines);
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
    let mut terminate_requested = false;
    sync_session_manifest(&args, endpoint_name.as_str(), &session_name_state, &session_bus);

    let exit_code = loop {
        drain_server_window_output(&session_bus, &output_rx);
        session_bus.poll_silence();
        if let Some(code) = handle_server_window_exit(
            &args,
            endpoint_name.as_str(),
            &session_name_state,
            &session_bus,
            &mut windows,
            &mut active_window,
        ) {
            session_bus.publish_exit(code);
            break code;
        }

        drain_server_controls(
            &args,
            endpoint_name.as_str(),
            &session_name_state,
            &session_bus,
            &control_rx,
            &output_tx,
            &mut windows,
            &mut active_window,
            &mut defaults,
            size,
            &mut terminate_requested,
        );

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