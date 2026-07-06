use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
};

const DEFAULT_LOG_PATH: &str = "screenlog.0";

pub(super) struct ScreenOutputLog {
    path: String,
    enabled: bool,
    file: Option<File>,
}

impl Default for ScreenOutputLog {
    fn default() -> Self {
        Self {
            path: DEFAULT_LOG_PATH.to_string(),
            enabled: false,
            file: None,
        }
    }
}

impl ScreenOutputLog {
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
        self.file = Some(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)?,
        );
        Ok(())
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
}