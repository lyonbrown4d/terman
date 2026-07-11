use std::{
    collections::BTreeMap,
    error::Error,
    io::Read,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
};

use crate::{
    ScreenArgs,
    pty_process::{ScreenPtyProcess, spawn_screen_pty},
    session_core::ScreenSessionBus,
};

pub(crate) struct ScreenWindowRuntime {
    index: Arc<AtomicUsize>,
    pub(crate) pty: ScreenPtyProcess,
}

impl ScreenWindowRuntime {
    pub(crate) fn index(&self) -> usize {
        self.index.load(Ordering::SeqCst)
    }

    pub(crate) fn set_index(&self, index: usize) {
        self.index.store(index, Ordering::SeqCst);
    }
}

pub(crate) struct ScreenWindowOutput {
    pub(crate) index: usize,
    pub(crate) bytes: Vec<u8>,
}

pub(crate) struct ScreenWindowExit {
    pub(crate) index: usize,
    pub(crate) code: i32,
}

pub(crate) enum ScreenWindowSwitch {
    Select(usize),
    Next,
    Previous,
    Last,
}

pub(crate) fn spawn_screen_window_runtime(
    args: &ScreenArgs,
    index: usize,
    command: Option<String>,
    cwd: Option<&Path>,
    env_overrides: &BTreeMap<String, Option<String>>,
    cols: u16,
    rows: u16,
    output_tx: mpsc::Sender<ScreenWindowOutput>,
) -> Result<ScreenWindowRuntime, Box<dyn Error>> {
    let mut window_args = args.clone();
    window_args.command = command;
    let mut pty = spawn_screen_pty(&window_args, cols, rows, cwd, env_overrides)?;
    let mut reader = pty.take_reader()?;
    let window_index = Arc::new(AtomicUsize::new(index));
    let output_index = window_index.clone();
    thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let message = ScreenWindowOutput {
                        index: output_index.load(Ordering::SeqCst),
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

    Ok(ScreenWindowRuntime { index: window_index, pty })
}

pub(crate) fn new_screen_window_title(
    command: Option<&str>,
    env_overrides: &BTreeMap<String, Option<String>>,
) -> Option<String> {
    command.map(str::to_string).or_else(|| {
        env_overrides
            .get("TERMAN_SCREEN_SHELL_TITLE")
            .and_then(Option::as_ref)
            .map(|title| title.trim())
            .filter(|title| !title.is_empty())
            .map(str::to_string)
    })
}

pub(crate) fn apply_default_window_log(
    bus: &ScreenSessionBus,
    env_overrides: &BTreeMap<String, Option<String>>,
) -> Result<(), Box<dyn Error>> {
    let enabled = env_overrides
        .get("TERMAN_SCREEN_DEFAULT_LOG")
        .and_then(Option::as_ref)
        .map(|state| matches!(state.as_str(), "on" | "1" | "true"))
        .unwrap_or(false);
    if enabled {
        bus.set_log_enabled(true)?;
    }
    Ok(())
}
pub(crate) fn next_screen_window_index(windows: &[ScreenWindowRuntime]) -> usize {
    windows
        .iter()
        .map(ScreenWindowRuntime::index)
        .max()
        .map(|index| index + 1)
        .unwrap_or(0)
}

pub(crate) fn renumber_screen_window(
    windows: &mut [ScreenWindowRuntime],
    source: usize,
    index: usize,
    active_window: &mut usize,
) -> bool {
    let Some(source_position) = windows.iter().position(|window| window.index() == source) else {
        return false;
    };
    if source == index {
        return true;
    }
    if let Some(target) = windows.iter().find(|window| window.index() == index) {
        target.set_index(source);
    }
    windows[source_position].set_index(index);
    if *active_window == source {
        *active_window = index;
    } else if *active_window == index {
        *active_window = source;
    }
    true
}

pub(crate) fn kill_active_window(windows: &mut [ScreenWindowRuntime], active_window: usize) {
    if let Some(window) = windows.iter_mut().find(|window| window.index() == active_window) {
        let _ = window.pty.kill();
    }
}

pub(crate) fn take_exited_window(windows: &mut Vec<ScreenWindowRuntime>) -> Option<ScreenWindowExit> {
    for position in 0..windows.len() {
        let code = match windows[position].pty.try_wait_code() {
            Ok(Some(code)) => Some(code),
            _ => None,
        };
        if let Some(code) = code {
            let window = windows.remove(position);
            return Some(ScreenWindowExit {
                index: window.index(),
                code,
            });
        }
    }
    None
}

pub(crate) fn switch_screen_window(
    bus: &ScreenSessionBus,
    windows: &[ScreenWindowRuntime],
    active_window: &mut usize,
    target: ScreenWindowSwitch,
) -> Option<Vec<u8>> {
    if windows.is_empty() {
        return None;
    }
    let active_position = windows
        .iter()
        .position(|window| window.index() == *active_window);
    let target_position = match target {
        ScreenWindowSwitch::Last => {
            let replay = bus.select_last_window()?;
            let status = bus.status_snapshot();
            if windows.iter().any(|window| window.index() == status.active_window) {
                *active_window = status.active_window;
                return Some(replay);
            }
            return None;
        }
        ScreenWindowSwitch::Select(index) => windows.iter().position(|window| window.index() == index)?,
        ScreenWindowSwitch::Next => active_position
            .map_or(0, |position| (position + 1) % windows.len()),
        ScreenWindowSwitch::Previous => active_position.map_or_else(
            || windows.len() - 1,
            |position| if position == 0 { windows.len() - 1 } else { position - 1 },
        ),
    };
    let index = windows[target_position].index();
    *active_window = index;
    bus.select_window(index)
}

pub(crate) fn write_active_window_input(
    windows: &mut [ScreenWindowRuntime],
    active_window: usize,
    bytes: &[u8],
) {
    if let Some(window) = windows.iter_mut().find(|window| window.index() == active_window) {
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