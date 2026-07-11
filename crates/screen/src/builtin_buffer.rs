use std::{fs, io, path::Path};

use crate::session_core::ScreenSessionBus;

pub(crate) fn read_buffer_file(bus: &ScreenSessionBus) {
    let path = bus.status_snapshot().buffer_file;
    match fs::read(&path) {
        Ok(bytes) => {
            let length = bytes.len();
            bus.set_paste_buffer(bytes);
            publish_message(
                bus,
                terman_common::builtin_screen_control_readbuf_complete_hint(
                    path_label(&path).as_str(),
                    length,
                ),
            );
        }
        Err(error) => publish_error(bus, &path, &error),
    }
}

pub(crate) fn write_buffer_file(bus: &ScreenSessionBus) {
    let path = bus.status_snapshot().buffer_file;
    let bytes = bus.paste_buffer_snapshot();
    match fs::write(&path, &bytes) {
        Ok(()) => publish_message(
            bus,
            terman_common::builtin_screen_control_writebuf_complete_hint(
                path_label(&path).as_str(),
                bytes.len(),
            ),
        ),
        Err(error) => publish_error(bus, &path, &error),
    }
}

pub(crate) fn remove_buffer_file(bus: &ScreenSessionBus) {
    let path = bus.status_snapshot().buffer_file;
    match fs::remove_file(&path) {
        Ok(()) => publish_removed(bus, &path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => publish_removed(bus, &path),
        Err(error) => publish_error(bus, &path, &error),
    }
}

fn publish_removed(bus: &ScreenSessionBus, path: &Path) {
    publish_message(
        bus,
        terman_common::builtin_screen_control_removebuf_complete_hint(
            path_label(path).as_str(),
        ),
    );
}

fn publish_error(bus: &ScreenSessionBus, path: &Path, error: &io::Error) {
    publish_message(
        bus,
        terman_common::builtin_screen_control_buffer_io_error_hint(
            path_label(path).as_str(),
            error.to_string().as_str(),
        ),
    );
}

fn publish_message(bus: &ScreenSessionBus, message: String) {
    bus.publish_message(format!("\r\n{message}\r\n").as_bytes());
}

fn path_label(path: &Path) -> String {
    path.display().to_string()
}
