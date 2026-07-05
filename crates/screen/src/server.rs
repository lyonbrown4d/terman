use std::{
    error::Error,
    io::{self, Read, Write},
    sync::mpsc,
    thread,
    time::Duration,
};

use portable_pty::{PtySize, native_pty_system};

use crate::{
    ScreenArgs,
    pty::build_command,
    service::ScreenSessionService,
    session_core::{ScreenControlEvent, ScreenSessionBus},
    sessions::register_builtin_screen_session,
};

pub(crate) fn run_screen_server(args: ScreenArgs) -> Result<(), Box<dyn Error>> {
    if args.session_name.is_none() {
        return Err(Box::new(io::Error::new(
            io::ErrorKind::InvalidInput,
            "internal screen server requires a session name",
        )));
    }

    let _session_record = register_builtin_screen_session(&args)?;
    let session_bus = ScreenSessionBus::new();
    let (control_tx, control_rx) = mpsc::channel::<ScreenControlEvent>();
    let _session_service = ScreenSessionService::start(
        args.session_name.as_deref(),
        session_bus.clone(),
        control_tx,
    )?;

    let pty_system = native_pty_system();
    let pty_size = PtySize {
        cols: args.cols.unwrap_or(120),
        rows: args.rows.unwrap_or(32),
        pixel_width: 0,
        pixel_height: 0,
    };

    let pair = pty_system.openpty(pty_size)?;
    let command = build_command(&args)?;
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

    let (exit_tx, exit_rx) = mpsc::channel::<i32>();
    let exit_bus = session_bus.clone();
    let child_wait_handle = thread::spawn(move || {
        let status = child
            .wait()
            .map(|status| status.exit_code() as i32)
            .unwrap_or(-1);
        exit_bus.publish_exit(status);
        let _ = exit_tx.send(status);
    });

    let mut exit_code = None;
    loop {
        match exit_rx.try_recv() {
            Ok(code) => {
                exit_code = Some(code);
                break;
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => break,
        }

        while let Ok(control) = control_rx.try_recv() {
            match control {
                ScreenControlEvent::Input(bytes) => {
                    if writer.write_all(&bytes).is_err() {
                        break;
                    }
                    if writer.flush().is_err() {
                        break;
                    }
                }
                ScreenControlEvent::Resize { cols, rows } => {
                    let size = PtySize {
                        cols,
                        rows,
                        pixel_width: 0,
                        pixel_height: 0,
                    };
                    let _ = master.resize(size);
                    session_bus.publish_resize(cols, rows);
                }
            }
        }

        thread::sleep(Duration::from_millis(16));
    }

    let _ = output_thread.join();
    let _ = child_wait_handle.join();

    let exit_code = exit_code.unwrap_or(-1);
    if exit_code == 0 {
        Ok(())
    } else {
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!("internal screen server exited with code {exit_code}"),
        )))
    }
}