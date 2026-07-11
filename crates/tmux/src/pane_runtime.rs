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
    window_view::TmuxWindowView,
};

pub(crate) struct TmuxPaneRuntimeConfig {
    pub(crate) session_name: String,
    pub(crate) window_index: u32,
    pub(crate) window_name: String,
    pub(crate) pane_index: u32,
    pub(crate) command: Option<String>,
    pub(crate) cols: u16,
    pub(crate) rows: u16,
    pub(crate) login_shell: bool,
}

pub(crate) struct TmuxPaneRuntime {
    index: u32,
    child: Box<dyn Child + Send + Sync>,
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    output_thread: Option<thread::JoinHandle<()>>,
}

impl TmuxPaneRuntime {
    pub(crate) fn spawn(
        config: TmuxPaneRuntimeConfig,
        view: Arc<Mutex<TmuxWindowView>>,
        bus: TmuxSessionBus,
    ) -> Result<Self, Box<dyn Error>> {
        let cols = config.cols.max(1);
        let rows = config.rows.max(1);
        let pair = native_pty_system().openpty(PtySize {
            cols,
            rows,
            pixel_width: 0,
            pixel_height: 0,
        })?;
        let command = build_tmux_pty_command(&TmuxPtyCommandSpec {
            session_name: config.session_name,
            window_index: config.window_index,
            window_name: config.window_name,
            pane_index: config.pane_index,
            command: config.command,
            login_shell: config.login_shell,
        });
        let child = pair.slave.spawn_command(command)?;
        let master = pair.master;
        let reader = master.try_clone_reader()?;
        let writer = master.take_writer()?;
        let output_thread = Some(spawn_output_thread(
            config.window_index,
            config.pane_index,
            reader,
            view,
            bus,
        ));
        Ok(Self {
            index: config.pane_index,
            child,
            master,
            writer,
            output_thread,
        })
    }

    pub(crate) fn index(&self) -> u32 {
        self.index
    }

    pub(crate) fn write_input(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()
    }

    pub(crate) fn resize(&self, cols: u16, rows: u16) {
        let _ = self.master.resize(PtySize {
            cols: cols.max(1),
            rows: rows.max(1),
            pixel_width: 0,
            pixel_height: 0,
        });
    }

    pub(crate) fn try_exit_code(&mut self) -> io::Result<Option<i32>> {
        self.child
            .try_wait()
            .map(|status| status.map(|status| status.exit_code() as i32))
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
    window_index: u32,
    pane_index: u32,
    mut reader: Box<dyn Read + Send>,
    view: Arc<Mutex<TmuxWindowView>>,
    bus: TmuxSessionBus,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let rendered = view
                        .lock()
                        .ok()
                        .and_then(|mut view| view.process_output(pane_index, &buf[..n]));
                    if let Some((active_pane, rendered)) = rendered {
                        bus.publish_window_frame(
                            window_index,
                            rendered.frame,
                            active_pane,
                            rendered.captures,
                        );
                    }
                }
                Err(_) => break,
            }
        }
    })
}
