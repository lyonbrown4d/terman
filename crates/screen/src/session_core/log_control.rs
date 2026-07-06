use std::io;

use super::ScreenSessionBus;

impl ScreenSessionBus {
    pub(crate) fn set_log_path(&self, path: String) -> io::Result<()> {
        let mut state = self
            .inner
            .lock()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;
        match state.active_window_mut() {
            Some(window) => window.set_log_path(path),
            None => Ok(()),
        }
    }

    pub(crate) fn set_log_enabled(&self, enabled: bool) -> io::Result<()> {
        let mut state = self
            .inner
            .lock()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;
        match state.active_window_mut() {
            Some(window) => window.set_log_enabled(enabled),
            None => Ok(()),
        }
    }

    pub(crate) fn toggle_log_enabled(&self) -> io::Result<()> {
        let mut state = self
            .inner
            .lock()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;
        match state.active_window_mut() {
            Some(window) => window.toggle_log_enabled(),
            None => Ok(()),
        }
    }
}