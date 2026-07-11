use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use crate::{
    ScreenArgs,
    server_manifest::sync_session_manifest,
    session_core::ScreenSessionBus,
    window_runtime::{ScreenWindowRuntime, ScreenWindowSwitch, switch_screen_window},
};

pub(super) fn switch_and_sync(
    args: &ScreenArgs,
    endpoint_name: &str,
    session_name_state: &Arc<Mutex<String>>,
    bus: &ScreenSessionBus,
    windows: &[ScreenWindowRuntime],
    active_window: &mut usize,
    target: ScreenWindowSwitch,
) {
    if let Some(replay) = switch_screen_window(bus, windows, active_window, target) {
        if bus.publish_region_redraw().is_none() {
            publish_window_redraw(bus, &replay);
        }
        sync_session_manifest(args, endpoint_name, session_name_state, bus);
    }
}

pub(super) fn sync_region_change(
    args: &ScreenArgs,
    endpoint_name: &str,
    session_name_state: &Arc<Mutex<String>>,
    bus: &ScreenSessionBus,
    active_window: &mut usize,
    result: Option<(usize, Vec<u8>)>,
) {
    if let Some((index, _)) = result {
        *active_window = index;
        sync_session_manifest(args, endpoint_name, session_name_state, bus);
    }
}

pub(super) fn publish_window_redraw(bus: &ScreenSessionBus, replay: &[u8]) {
    bus.publish_transient_output(b"\x1bc");
    if !replay.is_empty() {
        bus.publish_transient_output(replay);
    }
}

pub(super) fn publish_error(bus: &ScreenSessionBus, err: Box<dyn Error>) {
    let message = format!("\r\nscreen window failed: {err}\r\n");
    bus.publish_transient_output(message.as_bytes());
}