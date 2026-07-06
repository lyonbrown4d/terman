use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
};

const DEFAULT_LOG_PATH: &str = "screenlog.%n";

pub(super) struct ScreenOutputLog {
    path: String,
    window_index: usize,
    enabled: bool,
    file: Option<File>,
}

impl Default for ScreenOutputLog {
    fn default() -> Self {
        Self::new(0)
    }
}

impl ScreenOutputLog {
    pub(super) fn new(window_index: usize) -> Self {
        Self {
            path: DEFAULT_LOG_PATH.to_string(),
            window_index,
            enabled: false,
            file: None,
        }
    }

    pub(super) fn set_window_index(&mut self, window_index: usize) {
        if self.window_index != window_index {
            self.window_index = window_index;
            self.file = None;
        }
    }

    pub(super) fn set_path(&mut self, path: String) -> io::Result<()> {
        self.path = path;
        self.file = None;
        if self.enabled {
            self.open()?;
        }
        Ok(())
    }

    pub(super) fn set_enabled(&mut self, enabled: bool) -> io::Result<()> {
        self.enabled = enabled;
        if enabled {
            self.open()?;
        } else {
            self.file = None;
        }
        Ok(())
    }

    pub(super) fn toggle_enabled(&mut self) -> io::Result<()> {
        self.set_enabled(!self.enabled)
    }

    pub(super) fn append(&mut self, bytes: &[u8]) {
        if !self.enabled {
            return;
        }
        if self.file.is_none() && self.open().is_err() {
            self.enabled = false;
            return;
        }
        if let Some(file) = self.file.as_mut() {
            let result = file.write_all(bytes).and_then(|_| file.flush());
            if result.is_err() {
                self.enabled = false;
                self.file = None;
            }
        }
    }

    fn open(&mut self) -> io::Result<()> {
        let path = self.resolved_path();
        self.file = Some(OpenOptions::new().create(true).append(true).open(path)?);
        Ok(())
    }

    fn resolved_path(&self) -> String {
        self.path.replace("%n", &self.window_index.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::ScreenOutputLog;

    #[test]
    fn defaults_to_disabled_logging() {
        let mut log = ScreenOutputLog::default();
        log.append(b"ignored");
    }

    #[test]
    fn expands_window_number_in_log_path() {
        let mut log = ScreenOutputLog::new(3);
        assert_eq!(log.resolved_path(), "screenlog.3");
        log.set_window_index(12);
        assert_eq!(log.resolved_path(), "screenlog.12");
    }
}