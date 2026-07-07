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
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidInput,
                terman_common::builtin_tmux_internal_server_session_required_hint(),
            )));
        };
        let endpoint = args
            .internal_endpoint_name
            .as_deref()
            .map(TmuxIpcEndpoint::from_raw_name)
            .unwrap_or_else(|| TmuxIpcEndpoint::for_session(&session_name));

        Ok(Self {
            session_name,
            endpoint,
            cwd: current_tmux_cwd(),
            command: command_from_args(args.args),
            windows: 1,
            cols: 120,
            rows: 32,
            login_shell: false,
        })
    }
}

pub(crate) fn run_tmux_server(config: TmuxServerConfig) -> Result<(), Box<dyn Error>> {
    let session_name = Arc::new(Mutex::new(config.session_name.clone()));
    let _record_guard = SessionRecordGuard::new(session_name.clone());
    let session_bus = TmuxSessionBus::new(config.windows);
    let (control_tx, control_rx) = mpsc::channel::<TmuxControlEvent>();
    let _session_service = TmuxSessionService::start(
        session_name,
        config.endpoint.clone(),
        config.cwd.clone(),
        session_bus.clone(),
        control_tx,
    )?;

    session_bus.publish_resize(config.cols, config.rows);
    let mut active_window = TmuxWindowRuntime::spawn(
        TmuxWindowRuntimeConfig {
            session_name: config.session_name.clone(),
            index: 0,
            name: String::from("0"),
            command: config.command,
            cols: config.cols,
            rows: config.rows,
            login_shell: config.login_shell,
        },
        session_bus.clone(),
    )?;

    let exit_code = run_control_loop(&mut active_window, &session_bus, control_rx);
    active_window.join_output();

    if exit_code == 0 {
        Ok(())
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            terman_common::builtin_tmux_internal_server_exited_hint(exit_code),
        )))
    }
}

fn run_control_loop(
    active_window: &mut TmuxWindowRuntime,
    session_bus: &TmuxSessionBus,
    control_rx: mpsc::Receiver<TmuxControlEvent>,
) -> i32 {
    let mut terminate_requested = false;
    loop {
        match active_window.try_exit_code() {
            Ok(Some(code)) => {
                session_bus.publish_exit(code);
                return code;
            }
            Ok(None) => {}
            Err(_) => {
                session_bus.publish_exit(-1);
                return -1;
            }
        }

        while let Ok(control) = control_rx.try_recv() {
            match control {
                TmuxControlEvent::Input(bytes) => {
                    if active_window.write_input(&bytes).is_err() {
                        return -1;
                    }
                }
                TmuxControlEvent::Resize { cols, rows } => {
                    active_window.resize(cols, rows);
                    session_bus.publish_resize(cols, rows);
                }
                TmuxControlEvent::Terminate => {
                    if !terminate_requested {
                        terminate_requested = true;
                        active_window.kill();
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(16));
    }
}

fn command_from_args(args: Vec<String>) -> Option<String> {
    if args.is_empty() {
        None
    } else {
        Some(args.join(" "))
    }
}

fn current_tmux_cwd() -> String {
    env::current_dir()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|_| String::from("<unknown>"))
}

struct SessionRecordGuard {
    session_name: Arc<Mutex<String>>,
}

impl SessionRecordGuard {
    fn new(session_name: Arc<Mutex<String>>) -> Self {
        Self { session_name }
    }
}

impl Drop for SessionRecordGuard {
    fn drop(&mut self) {
        let Ok(session_name) = self.session_name.lock() else {
            return;
        };
        let _ = remove_builtin_tmux_session(&session_name);
    }
}