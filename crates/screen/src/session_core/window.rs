use std::io;

use super::{logging::ScreenOutputLog, replay::ScreenReplayBuffer, ScreenWindowStatus};

pub(super) struct ScreenWindowState {
    index: usize,
    title: Option<String>,
    replay: ScreenReplayBuffer,
    output_log: ScreenOutputLog,
}

impl ScreenWindowState {
    pub(super) fn new(index: usize) -> Self {
        Self {
            index,
            title: None,
            replay: ScreenReplayBuffer::default(),
            output_log: ScreenOutputLog::new(index),
        }
    }

    pub(super) fn index(&self) -> usize {
        self.index
    }

    pub(super) fn set_index(&mut self, index: usize) {
        self.index = index;
        self.output_log.set_window_index(index);
    }

    pub(super) fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub(super) fn set_title(&mut self, title: String) {
        self.title = Some(title);
    }

    pub(super) fn replay_snapshot(&self) -> Vec<u8> {
        self.replay.snapshot()
    }

    pub(super) fn replay_len(&self) -> usize {
        self.replay.len()
    }

    pub(super) fn scrollback_lines(&self) -> usize {
        self.replay.scrollback_lines()
    }

    pub(super) fn clear_replay(&mut self) {
        self.replay.clear();
    }

    pub(super) fn set_scrollback_lines(&mut self, lines: usize, cols: Option<u16>) {
        self.replay.set_scrollback_lines(lines, cols);
    }

    pub(super) fn set_log_path(&mut self, path: String) -> io::Result<()> {
        self.output_log.set_path(path)
    }

    pub(super) fn set_log_flush_interval(&mut self, seconds: u64) -> io::Result<()> {
        self.output_log.set_flush_interval(seconds)
    }

    pub(super) fn set_log_enabled(&mut self, enabled: bool) -> io::Result<()> {
        self.output_log.set_enabled(enabled)
    }

    pub(super) fn toggle_log_enabled(&mut self) -> io::Result<()> {
        self.output_log.toggle_enabled()
    }

    pub(super) fn append_output(&mut self, bytes: &[u8], cols: Option<u16>) {
        self.replay.append(bytes, cols);
        self.output_log.append(bytes);
    }

    pub(super) fn trim_to_cols(&mut self, cols: Option<u16>) {
        self.replay.trim_to_cols(cols);
    }

    pub(super) fn status(&self, active: bool) -> ScreenWindowStatus {
        ScreenWindowStatus {
            index: self.index,
            title: self.title.clone(),
            active,
            replay_bytes: self.replay.len(),
        }
    }
}