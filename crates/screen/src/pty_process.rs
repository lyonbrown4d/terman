use std::{collections::BTreeMap, error::Error, io::{self, Read, Write}, path::Path};

use portable_pty::{Child, MasterPty, PtySize, native_pty_system};

use crate::{ScreenArgs, pty::build_command};

pub(crate) struct ScreenPtyProcess {
    child: Box<dyn Child + Send + Sync>,
    master: Box<dyn MasterPty + Send>,
    reader: Option<Box<dyn Read + Send>>,
    writer: Box<dyn Write + Send>,
}

impl ScreenPtyProcess {
    pub(crate) fn take_reader(&mut self) -> io::Result<Box<dyn Read + Send>> {
        self.reader.take().ok_or_else(|| {
            io::Error::new(io::ErrorKind::BrokenPipe, "screen PTY reader was already taken")
        })
    }

    pub(crate) fn write_input(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()
    }

    pub(crate) fn resize(&self, cols: u16, rows: u16) {
        let _ = self.master.resize(pty_size(cols, rows));
    }

    pub(crate) fn try_wait_code(&mut self) -> Result<Option<i32>, Box<dyn Error>> {
        Ok(self.child.try_wait()?.map(|status| status.exit_code() as i32))
    }

    pub(crate) fn kill(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(self.child.kill()?)
    }

    pub(crate) fn wait_code(&mut self) -> Result<i32, Box<dyn Error>> {
        Ok(self.child.wait()?.exit_code() as i32)
    }
}

pub(crate) fn spawn_screen_pty(
    args: &ScreenArgs,
    cols: u16,
    rows: u16,
    cwd: Option<&Path>,
    env_overrides: &BTreeMap<String, Option<String>>,
) -> Result<ScreenPtyProcess, Box<dyn Error>> {
    let pty_system = native_pty_system();
    let pair = pty_system.openpty(pty_size(cols, rows))?;
    let command = build_command(args, cwd, env_overrides)?;
    let child = pair.slave.spawn_command(command)?;
    let master = pair.master;
    let reader = master.try_clone_reader()?;
    let writer = master.take_writer()?;

    Ok(ScreenPtyProcess {
        child,
        master,
        reader: Some(reader),
        writer,
    })
}

fn pty_size(cols: u16, rows: u16) -> PtySize {
    PtySize {
        cols,
        rows,
        pixel_width: 0,
        pixel_height: 0,
    }
}