use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::mpsc,
    thread,
    time::Duration,
};

use crate::{
    ScreenArgs,
    builtin_output::{publish_error, publish_window_redraw, write_region_frame},
    session_core::{ScreenControlEvent, ScreenSessionBus},
    window_runtime::{
        ScreenWindowOutput, ScreenWindowRuntime, ScreenWindowSwitch, apply_default_window_log,
        kill_active_window, kill_windows, new_screen_window_title, next_screen_window_index,
        renumber_screen_window, resize_windows, spawn_screen_window_runtime, switch_screen_window,
        write_active_window_input,
    },
};

pub(crate) struct BuiltinControlDefaults {
    pub(crate) cwd: Option<PathBuf>,
    pub(crate) env: BTreeMap<String, Option<String>>,
    scrollback_lines: usize,
}

impl BuiltinControlDefaults {
    pub(crate) fn new(scrollback_lines: usize) -> Self {
        Self { cwd: std::env::current_dir().ok(), env: BTreeMap::new(), scrollback_lines }
    }
}

pub(crate) fn drain_builtin_controls(
    args: &ScreenArgs,
    session_bus: &ScreenSessionBus,
    control_rx: &mpsc::Receiver<ScreenControlEvent>,
    output_tx: &mpsc::Sender<ScreenWindowOutput>,
    windows: &mut Vec<ScreenWindowRuntime>,
    active_window: &mut usize,
    defaults: &mut BuiltinControlDefaults,
    size: (u16, u16),
    display_output: bool,
) -> bool {
    while let Ok(control) = control_rx.try_recv() {
        if handle_control(
            args,
            session_bus,
            output_tx,
            windows,
            active_window,
            defaults,
            size,
            display_output,
            control,
        ) {
            thread::sleep(Duration::from_millis(16));
            return true;
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn handle_control(
    args: &ScreenArgs,
    session_bus: &ScreenSessionBus,
    output_tx: &mpsc::Sender<ScreenWindowOutput>,
    windows: &mut Vec<ScreenWindowRuntime>,
    active_window: &mut usize,
    defaults: &mut BuiltinControlDefaults,
    size: (u16, u16),
    display_output: bool,
    control: ScreenControlEvent,
) -> bool {
    match control {
        ScreenControlEvent::Input(bytes) => write_active_window_input(windows, *active_window, &bytes),
        ScreenControlEvent::BlankRegion => {
            if let Some((index, frame)) = session_bus.blank_region() {
                *active_window = index;
                if display_output { write_region_frame(&frame); }
            }
        }
        ScreenControlEvent::SetDefaultCwd { path } => defaults.cwd = Some(path),
        ScreenControlEvent::SetEnv { name, value } => { defaults.env.insert(name, Some(value)); }
        ScreenControlEvent::UnsetEnv { name } => { defaults.env.insert(name, None); }
        ScreenControlEvent::SetDefaultScrollback { lines } => defaults.scrollback_lines = lines,
        ScreenControlEvent::NewWindow { command } => {
            spawn_control_window(
                args,
                session_bus,
                output_tx,
                windows,
                active_window,
                defaults,
                size,
                display_output,
                command,
            );
        }
        ScreenControlEvent::SelectWindow { index } => switch_and_redraw(
            session_bus,
            windows,
            active_window,
            ScreenWindowSwitch::Select(index),
            display_output,
        ),
        ScreenControlEvent::NextWindow => switch_and_redraw(
            session_bus,
            windows,
            active_window,
            ScreenWindowSwitch::Next,
            display_output,
        ),
        ScreenControlEvent::PreviousWindow => switch_and_redraw(
            session_bus,
            windows,
            active_window,
            ScreenWindowSwitch::Previous,
            display_output,
        ),
        ScreenControlEvent::LastWindow => switch_and_redraw(
            session_bus,
            windows,
            active_window,
            ScreenWindowSwitch::Last,
            display_output,
        ),
        ScreenControlEvent::KillWindow => kill_active_window(windows, *active_window),
        ScreenControlEvent::NumberWindow { source, index } => {
            if renumber_screen_window(windows, source, index, active_window) {
                session_bus.renumber_window(source, index);
            }
        }
        ScreenControlEvent::SplitRegion { axis } => {
            if let Some((index, frame)) = session_bus.split_region(axis) {
                *active_window = index;
                if display_output { write_region_frame(&frame); }
            }
        }
        ScreenControlEvent::FocusRegion { target } => {
            if let Some((index, frame)) = session_bus.focus_region(target) {
                *active_window = index;
                if display_output { write_region_frame(&frame); }
            }
        }
        ScreenControlEvent::RemoveRegion => {
            if let Some((index, frame)) = session_bus.remove_region() {
                *active_window = index;
                if display_output { write_region_frame(&frame); }
            }
        }
        ScreenControlEvent::OnlyRegion => {
            if let Some((index, frame)) = session_bus.only_region() {
                *active_window = index;
                if display_output { write_region_frame(&frame); }
            }
        }
        ScreenControlEvent::ResizeRegion { resize } => {
            if let Some((index, frame)) = session_bus.resize_region(resize) {
                *active_window = index;
                if display_output { write_region_frame(&frame); }
            }
        }
        ScreenControlEvent::Resize { cols, rows } => {
            resize_windows(windows, cols, rows);
            session_bus.publish_resize(cols, rows);
            if let Some(frame) = session_bus.publish_region_redraw() {
                if display_output { write_region_frame(&frame); }
            }
        }
        ScreenControlEvent::Terminate => {
            kill_windows(windows);
            return true;
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn spawn_control_window(
    args: &ScreenArgs,
    session_bus: &ScreenSessionBus,
    output_tx: &mpsc::Sender<ScreenWindowOutput>,
    windows: &mut Vec<ScreenWindowRuntime>,
    active_window: &mut usize,
    defaults: &BuiltinControlDefaults,
    size: (u16, u16),
    display_output: bool,
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
            session_bus.add_window_with_scrollback(index, title, defaults.scrollback_lines);
            if let Err(err) = apply_default_window_log(session_bus, &defaults.env) {
                publish_error(session_bus, err);
            }
            windows.push(window);
            switch_and_redraw(
                session_bus,
                windows,
                active_window,
                ScreenWindowSwitch::Select(index),
                display_output,
            );
        }
        Err(err) => publish_error(session_bus, err),
    }
}

fn switch_and_redraw(
    session_bus: &ScreenSessionBus,
    windows: &[ScreenWindowRuntime],
    active_window: &mut usize,
    target: ScreenWindowSwitch,
    display_output: bool,
) {
    if let Some(replay) = switch_screen_window(session_bus, windows, active_window, target) {
        if let Some(frame) = session_bus.publish_region_redraw() {
            if display_output { write_region_frame(&frame); }
        } else {
            publish_window_redraw(session_bus, &replay, display_output);
        }
    }
}
