use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex, mpsc},
};

use crate::{
    ScreenArgs,
    server_manifest::sync_session_manifest,
    session_core::{ScreenControlEvent, ScreenSessionBus},
    window_runtime::{
        ScreenWindowOutput, ScreenWindowRuntime, ScreenWindowSwitch, apply_default_window_log,
        kill_active_window, kill_windows, new_screen_window_title, next_screen_window_index,
        renumber_screen_window, resize_windows, spawn_screen_window_runtime,
        take_exited_window, write_active_window_input,
    },
};

mod support;
use self::support::{publish_error, publish_window_redraw, switch_and_sync, sync_region_change};

pub(crate) struct ServerControlDefaults {
    pub(crate) cwd: Option<PathBuf>,
    pub(crate) env: BTreeMap<String, Option<String>>,
    scrollback_lines: usize,
}

impl ServerControlDefaults {
    pub(crate) fn new(scrollback_lines: usize) -> Self {
        Self {
            cwd: std::env::current_dir().ok(),
            env: BTreeMap::new(),
            scrollback_lines,
        }
    }
}

pub(crate) fn drain_server_window_output(
    bus: &ScreenSessionBus,
    rx: &mpsc::Receiver<ScreenWindowOutput>,
) {
    while let Ok(output) = rx.try_recv() {
        bus.publish_window_output(output.index, &output.bytes);
        bus.publish_region_redraw_for_output(output.index);
    }
}

pub(crate) fn handle_server_window_exit(
    args: &ScreenArgs,
    endpoint_name: &str,
    session_name_state: &Arc<Mutex<String>>,
    bus: &ScreenSessionBus,
    windows: &mut Vec<ScreenWindowRuntime>,
    active_window: &mut usize,
) -> Option<i32> {
    let exit = take_exited_window(windows)?;
    let removal = bus.remove_window(exit.index)?;
    if removal.last_window { return Some(exit.code); }
    if let Some(index) = removal.active_window {
        *active_window = index;
    }
    if removal.redraw && bus.publish_region_redraw().is_none() {
        publish_window_redraw(bus, &removal.replay);
    }
    sync_session_manifest(args, endpoint_name, session_name_state, bus);
    None
}

pub(crate) fn drain_server_controls(
    args: &ScreenArgs,
    endpoint_name: &str,
    session_name_state: &Arc<Mutex<String>>,
    bus: &ScreenSessionBus,
    control_rx: &mpsc::Receiver<ScreenControlEvent>,
    output_tx: &mpsc::Sender<ScreenWindowOutput>,
    windows: &mut Vec<ScreenWindowRuntime>,
    active_window: &mut usize,
    defaults: &mut ServerControlDefaults,
    size: (u16, u16),
    terminate_requested: &mut bool,
) {
    while let Ok(control) = control_rx.try_recv() {
        handle_control(
            args,
            endpoint_name,
            session_name_state,
            bus,
            output_tx,
            windows,
            active_window,
            defaults,
            size,
            terminate_requested,
            control,
        );
    }
}

fn handle_control(
    args: &ScreenArgs,
    endpoint_name: &str,
    session_name_state: &Arc<Mutex<String>>,
    bus: &ScreenSessionBus,
    output_tx: &mpsc::Sender<ScreenWindowOutput>,
    windows: &mut Vec<ScreenWindowRuntime>,
    active_window: &mut usize,
    defaults: &mut ServerControlDefaults,
    size: (u16, u16),
    terminate_requested: &mut bool,
    control: ScreenControlEvent,
) {
    match control {
        ScreenControlEvent::Input(bytes) => {
            write_active_window_input(windows, *active_window, &bytes)
        }
        ScreenControlEvent::SetDefaultCwd { path } => defaults.cwd = Some(path),
        ScreenControlEvent::SetEnv { name, value } => {
            defaults.env.insert(name, Some(value));
        }
        ScreenControlEvent::UnsetEnv { name } => {
            defaults.env.insert(name, None);
        }
        ScreenControlEvent::SetDefaultScrollback { lines } => defaults.scrollback_lines = lines,
        ScreenControlEvent::NewWindow { command } => spawn_control_window(
            args,
            endpoint_name,
            session_name_state,
            bus,
            output_tx,
            windows,
            active_window,
            defaults,
            size,
            command,
        ),
        ScreenControlEvent::SelectWindow { index } => switch_and_sync(
            args,
            endpoint_name,
            session_name_state,
            bus,
            windows,
            active_window,
            ScreenWindowSwitch::Select(index),
        ),
        ScreenControlEvent::NextWindow => switch_and_sync(
            args,
            endpoint_name,
            session_name_state,
            bus,
            windows,
            active_window,
            ScreenWindowSwitch::Next,
        ),
        ScreenControlEvent::PreviousWindow => switch_and_sync(
            args,
            endpoint_name,
            session_name_state,
            bus,
            windows,
            active_window,
            ScreenWindowSwitch::Previous,
        ),
        ScreenControlEvent::LastWindow => switch_and_sync(
            args,
            endpoint_name,
            session_name_state,
            bus,
            windows,
            active_window,
            ScreenWindowSwitch::Last,
        ),
        ScreenControlEvent::KillWindow => kill_active_window(windows, *active_window),
        ScreenControlEvent::NumberWindow { source, index } => {
            if renumber_screen_window(windows, source, index, active_window) {
                bus.renumber_window(source, index);
                bus.publish_region_redraw();
                sync_session_manifest(args, endpoint_name, session_name_state, bus);
            }
        }
        ScreenControlEvent::SplitRegion { axis } => sync_region_change(
            args,
            endpoint_name,
            session_name_state,
            bus,
            active_window,
            bus.split_region(axis),
        ),
        ScreenControlEvent::FocusRegion { target } => sync_region_change(
            args,
            endpoint_name,
            session_name_state,
            bus,
            active_window,
            bus.focus_region(target),
        ),
        ScreenControlEvent::RemoveRegion => sync_region_change(
            args,
            endpoint_name,
            session_name_state,
            bus,
            active_window,
            bus.remove_region(),
        ),
        ScreenControlEvent::OnlyRegion => sync_region_change(
            args,
            endpoint_name,
            session_name_state,
            bus,
            active_window,
            bus.only_region(),
        ),
        ScreenControlEvent::Resize { cols, rows } => {
            resize_windows(windows, cols, rows);
            bus.publish_resize(cols, rows);
            bus.publish_region_redraw();
            sync_session_manifest(args, endpoint_name, session_name_state, bus);
        }
        ScreenControlEvent::Terminate => {
            if !*terminate_requested {
                *terminate_requested = true;
                kill_windows(windows);
            }
        }
    }
}

fn spawn_control_window(
    args: &ScreenArgs,
    endpoint_name: &str,
    session_name_state: &Arc<Mutex<String>>,
    bus: &ScreenSessionBus,
    output_tx: &mpsc::Sender<ScreenWindowOutput>,
    windows: &mut Vec<ScreenWindowRuntime>,
    active_window: &mut usize,
    defaults: &ServerControlDefaults,
    size: (u16, u16),
    command: Option<String>,
) {
    let index = next_screen_window_index(windows);
    let title = new_screen_window_title(command.as_deref(), &defaults.env);
    match spawn_screen_window_runtime(
        args,
        index,
        command,
        defaults.cwd.as_deref(),
        &defaults.env,
        size.0,
        size.1,
        output_tx.clone(),
    ) {
        Ok(window) => {
            bus.add_window_with_scrollback(index, title, defaults.scrollback_lines);
            if let Err(err) = apply_default_window_log(bus, &defaults.env) {
                publish_error(bus, err);
            }
            windows.push(window);
            switch_and_sync(
                args,
                endpoint_name,
                session_name_state,
                bus,
                windows,
                active_window,
                ScreenWindowSwitch::Select(index),
            );
        }
        Err(err) => publish_error(bus, err),
    }
}

