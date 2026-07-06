use std::{
    fs::{File, OpenOptions},
    io::{self, Write},
    time::{Duration, Instant},
};

use chrono::{Datelike, Local, Timelike};

const DEFAULT_LOG_PATH: &str = "screenlog.%n";
const DEFAULT_FLUSH_SECONDS: u64 = 10;
const DEFAULT_TIMESTAMP_SECONDS: u64 = 120;
const DEFAULT_TIMESTAMP_FORMAT: &str = "-- %n:%t -- time-stamp -- %M/%d/%y %c:%s --\n";

pub(super) struct ScreenOutputLog {
    path: String,
    window_index: usize,
    flush_interval: Duration,
    timestamp_enabled: bool,
    timestamp_after: Duration,
    timestamp_format: String,
    last_output: Option<Instant>,
    last_flush: Instant,
    dirty: bool,
    enabled: bool,
    file: Option<File>,
}

impl Default for ScreenOutputLog {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Drop for ScreenOutputLog {
    fn drop(&mut self) {
        let _ = self.flush_pending();
    }
}

impl ScreenOutputLog {
    pub(super) fn new(window_index: usize) -> Self {
        Self {
            path: DEFAULT_LOG_PATH.to_string(),
            window_index,
            flush_interval: Duration::from_secs(DEFAULT_FLUSH_SECONDS),
            timestamp_enabled: false,
            timestamp_after: Duration::from_secs(DEFAULT_TIMESTAMP_SECONDS),
            timestamp_format: DEFAULT_TIMESTAMP_FORMAT.to_string(),
            last_output: None,
            last_flush: Instant::now(),
            dirty: false,
            enabled: false,
            file: None,
        }
    }

    pub(super) fn set_window_index(&mut self, window_index: usize) {
        if self.window_index != window_index {
            let _ = self.flush_pending();
            self.window_index = window_index;
            self.file = None;
        }
    }

    pub(super) fn set_path(&mut self, path: String) -> io::Result<()> {
        self.flush_pending()?;
        self.path = path;
        self.file = None;
        if self.enabled {
            self.open()?;
        }
        Ok(())
    }

    pub(super) fn set_flush_interval(&mut self, seconds: u64) -> io::Result<()> {
        self.flush_pending()?;
        self.flush_interval = Duration::from_secs(seconds);
        Ok(())
    }

    pub(super) fn set_timestamp_enabled(&mut self, enabled: bool) {
        self.timestamp_enabled = enabled;
    }

    pub(super) fn toggle_timestamp_enabled(&mut self) {
        self.timestamp_enabled = !self.timestamp_enabled;
    }

    pub(super) fn set_timestamp_after(&mut self, seconds: u64) {
        self.timestamp_after = Duration::from_secs(seconds);
    }

    pub(super) fn set_timestamp_format(&mut self, value: String) {
        self.timestamp_format = value;
    }

    pub(super) fn set_enabled(&mut self, enabled: bool) -> io::Result<()> {
        self.enabled = enabled;
        if enabled {
            self.open()?;
        } else {
            self.flush_pending()?;
            self.file = None;
            self.last_output = None;
        }
        Ok(())
    }

    pub(super) fn toggle_enabled(&mut self) -> io::Result<()> {
        self.set_enabled(!self.enabled)
    }

    pub(super) fn append(&mut self, bytes: &[u8], title: Option<&str>) {
        if !self.enabled {
            return;
        }
        if self.file.is_none() && self.open().is_err() {
            self.enabled = false;
            return;
        }
        if self.timestamp_due() {
            let stamp = self.timestamp_text(title);
            if self.write_bytes(stamp.as_bytes()).is_err() {
                return;
            }
        }
        if self.write_bytes(bytes).is_ok() {
            self.last_output = Some(Instant::now());
        }
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        let Some(file) = self.file.as_mut() else {
            return Ok(());
        };
        if file.write_all(bytes).is_err() {
            self.enabled = false;
            self.file = None;
            return Err(io::Error::new(io::ErrorKind::Other, "screen log write failed"));
        }
        self.dirty = true;
        if self.should_flush() && self.flush_pending().is_err() {
            self.enabled = false;
            self.file = None;
            return Err(io::Error::new(io::ErrorKind::Other, "screen log flush failed"));
        }
        Ok(())
    }

    fn timestamp_due(&self) -> bool {
        self.timestamp_enabled
            && self
                .last_output
                .map(|last| last.elapsed() >= self.timestamp_after)
                .unwrap_or(false)
    }

    fn timestamp_text(&self, title: Option<&str>) -> String {
        let now = Local::now();
        self.timestamp_format
            .replace("%n", &self.window_index.to_string())
            .replace("%t", title.unwrap_or(""))
            .replace("%M", &format!("{:02}", now.month()))
            .replace("%d", &format!("{:02}", now.day()))
            .replace("%y", &format!("{:02}", now.year().rem_euclid(100)))
            .replace("%c", &format!("{:02}", now.hour()))
            .replace("%s", &format!("{:02}", now.second()))
            .replace("\\n", "\n")
    }

    fn should_flush(&self) -> bool {
        self.flush_interval.is_zero() || self.last_flush.elapsed() >= self.flush_interval
    }

    fn flush_pending(&mut self) -> io::Result<()> {
        if self.dirty {
            if let Some(file) = self.file.as_mut() {
                file.flush()?;
            }
            self.dirty = false;
        }
        self.last_flush = Instant::now();
        Ok(())
    }

    fn open(&mut self) -> io::Result<()> {
        let path = self.resolved_path();
        self.file = Some(OpenOptions::new().create(true).append(true).open(path)?);
        self.last_flush = Instant::now();
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
        log.append(b"ignored", None);
    }

    #[test]
    fn expands_window_number_in_log_path() {
        let mut log = ScreenOutputLog::new(3);
        assert_eq!(log.resolved_path(), "screenlog.3");
        log.set_window_index(12);
        assert_eq!(log.resolved_path(), "screenlog.12");
    }

    #[test]
    fn accepts_flush_interval_changes() {
        let mut log = ScreenOutputLog::new(0);
        log.set_flush_interval(0).unwrap();
        assert!(log.flush_interval.is_zero());
    }

    #[test]
    fn expands_timestamp_tokens() {
        let log = ScreenOutputLog::new(7);
        let stamp = log.timestamp_text(Some("editor"));
        assert!(stamp.contains("7:editor"));
    }
}