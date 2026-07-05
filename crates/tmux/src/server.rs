use std::{
    env,
    error::Error,
    io::{self, Read, Write},
    sync::mpsc,
    thread,
    time::Duration,
};

use portable_pty::{PtySize, native_pty_system};

use crate::{
    TmuxArgs,
    ipc::TmuxIpcEndpoint,
    pty::{TmuxPtyCommandSpec, build_tmux_pty_command},
    service::TmuxSessionService,
    session_core::{TmuxControlEvent, TmuxSessionBus},
    sessions::remove_builtin_tmux_session,
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
                "internal tmux server requires a session name",
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
    let _record_guard = SessionRecordGuard::new(config.session_name.clone());
    let session_bus = TmuxSessionBus::new(config.windows);
    let (control_tx, control_rx) = mpsc::channel::<TmuxControlEvent>();
    let _session_service = TmuxSessionService::start(
        &config.session_name,
        config.endpoint.clone(),
        config.cwd.clone(),
        session_bus.clone(),
        control_tx,
    )?;

    let pty_system = native_pty_system();
    let pty_size = PtySize {
        cols: config.cols,
        rows: config.rows,
        pixel_width: 0,
        pixel_height: 0,
    };
    session_bus.publish_resize(config.cols, config.rows);

    let pair = pty_system.openpty(pty_size)?;
    let command = build_tmux_pty_command(&TmuxPtyCommandSpec {
        session_name: config.session_name.clone(),
        window_index: 0,
        window_name: String::from("0"),
        command: config.command,
        login_shell: config.login_shell,
    });
    let mut child = pair.slave.spawn_command(command)?;

    let master = pair.master;
    let mut reader = master.try_clone_reader()?;
    let mut writer = master.take_writer()?;

    let output_bus = session_bus.clone();
    let output_thread = thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => output_bus.publish_output(&buf[..n]),
                Err(_) => break,
            }
        }
    });

    let exit_code = run_control_loop(&mut child, &master, &mut writer, &session_bus, control_rx);
    let _ = output_thread.join();

    if exit_code == 0 {
        Ok(())
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("internal tmux server exited with code {exit_code}"),
        )))
    }
}

fn run_control_loop(
    child: &mut Box<dyn portable_pty::Child + Send + Sync>,
    master: &Box<dyn portable_pty::MasterPty + Send>,
    writer: &mut Box<dyn Write + Send>,
    session_bus: &TmuxSessionBus,
    control_rx: mpsc::Receiver<TmuxControlEvent>,
) -> i32 {
    let mut terminate_requested = false;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let code = status.exit_code() as i32;
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
                    if writer.write_all(&bytes).is_err() || writer.flush().is_err() {
                        return -1;
                    }
                }
                TmuxControlEvent::Resize { cols, rows } => {
                    let size = PtySize {
                        cols,
                        rows,
                        pixel_width: 0,
                        pixel_height: 0,
                    };
                    let _ = master.resize(size);
                    session_bus.publish_resize(cols, rows);
                }
                TmuxControlEvent::Terminate => {
                    if !terminate_requested {
                        terminate_requested = true;
                        let _ = child.kill();
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
    session_name: String,
}

impl SessionRecordGuard {
    fn new(session_name: String) -> Self {
        Self { session_name }
    }
}

impl Drop for SessionRecordGuard {
    fn drop(&mut self) {
        let _ = remove_builtin_tmux_session(&self.session_name);
    }
}