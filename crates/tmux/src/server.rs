use std::{
    env,
    error::Error,
    io,
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

use crate::{
    TmuxArgs,
    ipc::TmuxIpcEndpoint,
    service::TmuxSessionService,
    session_core::{TmuxControlEvent, TmuxSessionBus},
    sessions::remove_builtin_tmux_session,
    window_runtime::{TmuxWindowRuntime, TmuxWindowRuntimeConfig},
};

pub(crate) struct TmuxServerConfig {
    session_name: String,
    endpoint: TmuxIpcEndpoint,
    cwd: String,
    command: Option<String>,
    windows: u32,
    cols: u16,
    rows: u16,
    login_shell: bool,
}

impl TmuxServerConfig {
    pub(crate) fn from_args(args: TmuxArgs) -> Result<Self, Box<dyn Error>> {
        let Some(session_name) = args.internal_session_name else {
            return Err(Box::new(io::Error::new(io::ErrorKind::InvalidInput, terman_common::builtin_tmux_internal_server_session_required_hint())));
        };
        let endpoint = args.internal_endpoint_name.as_deref().map(TmuxIpcEndpoint::from_raw_name).unwrap_or_else(|| TmuxIpcEndpoint::for_session(&session_name));
        Ok(Self { session_name, endpoint, cwd: current_tmux_cwd(), command: command_from_args(args.args), windows: 1, cols: 120, rows: 32, login_shell: false })
    }
}

pub(crate) fn run_tmux_server(config: TmuxServerConfig) -> Result<(), Box<dyn Error>> {
    let session_name = Arc::new(Mutex::new(config.session_name.clone()));
    let _record_guard = SessionRecordGuard::new(session_name.clone());
    let session_bus = TmuxSessionBus::new(config.windows);
    let (control_tx, control_rx) = mpsc::channel::<TmuxControlEvent>();
    let _session_service = TmuxSessionService::start(session_name.clone(), config.endpoint.clone(), config.cwd.clone(), session_bus.clone(), control_tx)?;
    session_bus.publish_resize(config.cols, config.rows);
    let mut windows = vec![spawn_window(config.session_name.clone(), 0, String::from("0"), config.command, config.cols, config.rows, config.login_shell, &session_bus)?];
    let mut active_window = 0;
    let exit_code = run_control_loop(&mut windows, &mut active_window, &session_name, config.login_shell, &session_bus, control_rx);
    for window in &mut windows {
        window.join_output();
    }
    if exit_code == 0 { Ok(()) } else { Err(Box::new(io::Error::new(io::ErrorKind::Other, terman_common::builtin_tmux_internal_server_exited_hint(exit_code)))) }
}

fn run_control_loop(
    windows: &mut Vec<TmuxWindowRuntime>,
    active_window: &mut u32,
    session_name: &Arc<Mutex<String>>,
    login_shell: bool,
    session_bus: &TmuxSessionBus,
    control_rx: mpsc::Receiver<TmuxControlEvent>,
) -> i32 {
    let mut terminate_requested = false;
    let mut cols = 120;
    let mut rows = 32;
    loop {
        if let Some(code) = handle_exited_window(windows, active_window, session_bus) {
            return code;
        }
        while let Ok(control) = control_rx.try_recv() {
            match control {
                TmuxControlEvent::Input(bytes) => {
                    if active_runtime(windows, *active_window).and_then(|window| window.write_input(&bytes).ok()).is_none() { return -1; }
                }
                TmuxControlEvent::Resize { cols: next_cols, rows: next_rows } => {
                    cols = next_cols;
                    rows = next_rows;
                    for window in windows.iter_mut() { window.resize(cols, rows); }
                    session_bus.publish_resize(cols, rows);
                }
                TmuxControlEvent::NewWindow { index, name, command } => {
                    if active_runtime(windows, index).is_none() {
                        let session = current_session_name(session_name);
                        match spawn_window(session, index, name.clone(), command, cols, rows, login_shell, session_bus) {
                            Ok(window) => { windows.push(window); session_bus.add_window(index, name); }
                            Err(err) => publish_error(session_bus, err),
                        }
                    }
                    if active_runtime(windows, index).is_some() && session_bus.select_window(index) { *active_window = index; }
                }
                TmuxControlEvent::RenameWindow { index, name } => {
                    if let Some(window) = active_runtime(windows, index) {
                        window.rename(name.clone());
                        let _ = session_bus.rename_window(index, name);
                    }
                }                TmuxControlEvent::KillWindow { index } => {
                    if let Some(window) = active_runtime(windows, index) {
                        window.kill();
                    }
                }
                TmuxControlEvent::SelectWindow { index } => {
                    if active_runtime(windows, index).is_some() { *active_window = index; }
                }
                TmuxControlEvent::SplitPane { window, horizontal, command } => {
                    if let Some(runtime) = active_runtime(windows, window) {
                        if let Err(err) = runtime.split(horizontal, command) { publish_error(session_bus, err); }
                    }
                }
                TmuxControlEvent::SelectPane { window, pane } => {
                    if let Some(runtime) = active_runtime(windows, window) {
                        if runtime.select_pane(pane) && *active_window != window {
                            *active_window = window;
                            let _ = session_bus.select_window(window);
                        }
                    }
                }
                TmuxControlEvent::SwapPane { window, source, target } => {
                    if let Some(runtime) = active_runtime(windows, window) {
                        let _ = runtime.swap_panes(source, target);
                    }
                }
                TmuxControlEvent::KillPane { window, pane } => {
                    if let Some(runtime) = active_runtime(windows, window) { let _ = runtime.kill_pane(pane); }
                }
                TmuxControlEvent::TogglePaneZoom { window, pane } => {
                    if let Some(runtime) = active_runtime(windows, window) {
                        let _ = runtime.toggle_pane_zoom(pane);
                    }
                }
                TmuxControlEvent::SetSynchronizePanes { window, enabled } => {
                    if let Some(runtime) = active_runtime(windows, window) {
                        runtime.set_synchronize_panes(enabled);
                    }
                }
                TmuxControlEvent::ResizePane { window, pane, cols, rows } => {
                    if let Some(runtime) = active_runtime(windows, window) { let _ = runtime.resize_pane(pane, cols, rows); }
                }
                TmuxControlEvent::Terminate => {
                    if !terminate_requested {
                        terminate_requested = true;
                        for window in windows.iter_mut() { window.kill(); }
                    }
                }
            }
        }
        thread::sleep(Duration::from_millis(16));
    }
}

fn handle_exited_window(windows: &mut Vec<TmuxWindowRuntime>, active_window: &mut u32, session_bus: &TmuxSessionBus) -> Option<i32> {
    let mut exited = None;
    for (position, window) in windows.iter_mut().enumerate() {
        match window.try_exit_code() {
            Ok(Some(code)) => { exited = Some((position, code)); break; }
            Ok(None) => {}
            Err(_) => { session_bus.publish_exit(-1); return Some(-1); }
        }
    }
    let (position, code) = exited?;
    let mut window = windows.remove(position);
    let index = window.index();
    window.join_output();
    session_bus.remove_window(index);
    if windows.is_empty() {
        session_bus.publish_exit(code);
        return Some(code);
    }
    let next_position = position.min(windows.len() - 1);
    *active_window = windows[next_position].index();
    let _ = session_bus.select_window(*active_window);
    None
}

fn active_runtime(windows: &mut [TmuxWindowRuntime], index: u32) -> Option<&mut TmuxWindowRuntime> {
    windows.iter_mut().find(|window| window.index() == index)
}

fn spawn_window(session_name: String, index: u32, name: String, command: Option<String>, cols: u16, rows: u16, login_shell: bool, session_bus: &TmuxSessionBus) -> Result<TmuxWindowRuntime, Box<dyn Error>> {
    TmuxWindowRuntime::spawn(TmuxWindowRuntimeConfig { session_name, index, name, command, cols, rows, login_shell }, session_bus.clone())
}

fn current_session_name(session_name: &Arc<Mutex<String>>) -> String {
    session_name.lock().map(|name| name.clone()).unwrap_or_else(|_| String::from("tmux"))
}

fn publish_error(bus: &TmuxSessionBus, err: Box<dyn Error>) {
    let message = format!("\r\ntmux window failed: {err}\r\n");
    bus.publish_transient_output(message.as_bytes());
}

fn command_from_args(args: Vec<String>) -> Option<String> {
    if args.is_empty() { None } else { Some(args.join(" ")) }
}

fn current_tmux_cwd() -> String {
    env::current_dir().map(|path| path.to_string_lossy().to_string()).unwrap_or_else(|_| String::from("<unknown>"))
}

struct SessionRecordGuard {
    session_name: Arc<Mutex<String>>,
}

impl SessionRecordGuard {
    fn new(session_name: Arc<Mutex<String>>) -> Self { Self { session_name } }
}

impl Drop for SessionRecordGuard {
    fn drop(&mut self) {
        let Ok(session_name) = self.session_name.lock() else { return; };
        let _ = remove_builtin_tmux_session(&session_name);
    }
}