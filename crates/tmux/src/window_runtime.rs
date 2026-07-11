use std::{
    error::Error,
    io::{self, Read, Write},
    sync::{Arc, Mutex},
    thread,
};

use portable_pty::{Child, MasterPty, PtySize, native_pty_system};

use crate::{
    pty::{TmuxPtyCommandSpec, build_tmux_pty_command},
    session_core::TmuxSessionBus,
    terminal_frame::render_terminal,
};

pub(crate) struct TmuxWindowRuntimeConfig {
    pub(crate) session_name: String,
    pub(crate) index: u32,
    pub(crate) name: String,
    pub(crate) command: Option<String>,
    pub(crate) cols: u16,
    pub(crate) rows: u16,
    pub(crate) login_shell: bool,
}

pub(crate) struct TmuxWindowRuntime {
    index: u32,
    name: String,
    child: Box<dyn Child + Send + Sync>,
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    terminal: Arc<Mutex<vt100::Parser>>,
    bus: TmuxSessionBus,
    output_thread: Option<thread::JoinHandle<()>>,
}

impl TmuxWindowRuntime {
    pub(crate) fn spawn(config: TmuxWindowRuntimeConfig, bus: TmuxSessionBus) -> Result<Self, Box<dyn Error>> {
        let index = config.index;
        let name = config.name.clone();
        let cols = config.cols.max(1);
        let rows = config.rows.max(1);
        let terminal = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, 10_000)));
        let pair = native_pty_system().openpty(PtySize {
            cols,
            rows,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        let command = build_tmux_pty_command(&TmuxPtyCommandSpec {
            session_name: config.session_name,
            window_index: index,
            window_name: config.name,
            command: config.command,
            login_shell: config.login_shell,
        });
        let child = pair.slave.spawn_command(command)?;
        let master = pair.master;
        let reader = master.try_clone_reader()?;
        let writer = master.take_writer()?;
        let output_thread = Some(spawn_output_thread(
            index,
            reader,
            terminal.clone(),
            bus.clone(),
        ));
        Ok(Self {
            index,
            name,
            child,
            master,
            writer,
            terminal,
            bus,
            output_thread,
        })
    }

    pub(crate) fn index(&self) -> u32 {
        self.index
    }

    pub(crate) fn rename(&mut self, name: String) {
        self.name = name;
    }

    pub(crate) fn write_input(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()
    }

    pub(crate) fn resize(&self, cols: u16, rows: u16) {
        let cols = cols.max(1);
        let rows = rows.max(1);
        let _ = self.master.resize(PtySize {
            cols,
            rows,
            pixel_width: 0,
            pixel_height: 0,
        });
        let rendered = {
            let Ok(mut terminal) = self.terminal.lock() else { return; };
            terminal.screen_mut().set_size(rows, cols);
            render_terminal(terminal.screen())
        };
        self.bus
            .publish_window_frame(self.index, rendered.frame, rendered.capture);
    }

    pub(crate) fn try_exit_code(&mut self) -> io::Result<Option<i32>> {
        self.child.try_wait().map(|status| status.map(|status| status.exit_code() as i32))
    }

    pub(crate) fn kill(&mut self) {
        let _ = self.child.kill();
    }

    pub(crate) fn join_output(&mut self) {
        if let Some(handle) = self.output_thread.take() {
            let _ = handle.join();
        }
    }
}

fn spawn_output_thread(
    index: u32,
    mut reader: Box<dyn Read + Send>,
    terminal: Arc<Mutex<vt100::Parser>>,
    bus: TmuxSessionBus,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let rendered = {
                        let Ok(mut terminal) = terminal.lock() else { break; };
                        terminal.process(&buf[..n]);
                        render_terminal(terminal.screen())
                    };
                    bus.publish_window_frame(index, rendered.frame, rendered.capture);
                }
                Err(_) => break,
            }
        }
    })
}