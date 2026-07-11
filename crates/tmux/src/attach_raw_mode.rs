use std::io;

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

use crate::attach_mouse::{disable_mouse_capture, enable_mouse_capture};

pub(super) struct RawModeGuard;

impl RawModeGuard {
    pub(super) fn enable() -> io::Result<Self> {
        enable_raw_mode()?;
        enable_mouse_capture()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        disable_mouse_capture();
        let _ = disable_raw_mode();
    }
}