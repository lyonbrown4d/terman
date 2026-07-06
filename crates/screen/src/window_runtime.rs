use std::{error::Error, io::Read, sync::mpsc, thread};

use crate::{ScreenArgs, pty_process::{ScreenPtyProcess, spawn_screen_pty}};

pub(crate) struct ScreenWindowRuntime {
    pub(crate) index: usize,
    pub(crate) pty: ScreenPtyProcess,
}

pub(crate) struct ScreenWindowOutput {
    pub(crate) index: usize,
    pub(crate) bytes: Vec<u8>,
}

pub(crate) fn spawn_screen_window_runtime(
    args: &ScreenArgs,
    index: usize,
    command: Option<String>,
    cols: u16,
    rows: u16,
    output_tx: mpsc::Sender<ScreenWindowOutput>,
) -> Result<ScreenWindowRuntime, Box<dyn Error>> {
    let mut window_args = args.clone();
    window_args.command = command;
    let mut pty = spawn_screen_pty(&window_args, cols, rows)?;
    let mut reader = pty.take_reader()?;
    thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let message = ScreenWindowOutput {
                        index,
                        bytes: buf[..n].to_vec(),
                    };
                    if output_tx.send(message).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    Ok(ScreenWindowRuntime { index, pty })
}

pub(crate) fn write_active_window_input(
    windows: &mut [ScreenWindowRuntime],
    active_window: usize,
    bytes: &[u8],
) {
    if let Some(window) = windows.iter_mut().find(|window| window.index == active_window) {
        let _ = window.pty.write_input(bytes);
    }
}

pub(crate) fn resize_windows(windows: &[ScreenWindowRuntime], cols: u16, rows: u16) {
    for window in windows {
        window.pty.resize(cols, rows);
    }
}

pub(crate) fn kill_windows(windows: &mut [ScreenWindowRuntime]) {
    for window in windows {
        let _ = window.pty.kill();
    }
}