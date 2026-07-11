use std::{
    error::Error,
    io::{self, Write},
    sync::mpsc,
};

use crate::{
    session_core::ScreenSessionBus,
    window_runtime::{ScreenWindowOutput, ScreenWindowRuntime, take_exited_window},
};

pub(crate) fn publish_window_redraw(bus: &ScreenSessionBus, replay: &[u8]) {
    bus.publish_transient_output(b"c");
    if !replay.is_empty() {
        bus.publish_transient_output(replay);
    }
    let mut bytes = b"c".to_vec();
    bytes.extend_from_slice(replay);
    write_region_frame(&bytes);
}

pub(crate) fn write_region_frame(frame: &[u8]) {
    let mut stdout = io::stdout();
    let _ = stdout.write_all(frame);
    let _ = stdout.flush();
}

pub(crate) fn drain_window_output(
    bus: &ScreenSessionBus,
    rx: &mpsc::Receiver<ScreenWindowOutput>,
    active_window: usize,
) {
    while let Ok(output) = rx.try_recv() {
        bus.publish_window_output(output.index, &output.bytes);
        if let Some(frame) = bus.publish_region_redraw_for_output(output.index) {
            write_region_frame(&frame);
        } else if output.index == active_window {
            write_region_frame(&output.bytes);
        }
    }
}

pub(crate) fn handle_window_exit(
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
        if let Some(frame) = bus.publish_region_redraw() {
            write_region_frame(&frame);
        } else {
            publish_window_redraw(bus, &removal.replay);
        }
    }
    None
}

pub(crate) fn publish_error(bus: &ScreenSessionBus, err: Box<dyn Error>) {
    let message = format!("
screen window failed: {err}
");
    bus.publish_transient_output(message.as_bytes());
}