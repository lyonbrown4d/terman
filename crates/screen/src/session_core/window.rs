use std::io;

use super::{logging::ScreenOutputLog, replay::ScreenReplayBuffer, ScreenWindowStatus};

const DEFAULT_TERMINAL_ROWS: u16 = 24;
const DEFAULT_TERMINAL_COLS: u16 = 80;

pub(super) struct ScreenWindowState {
    index: usize,
    title: Option<String>,
    replay: ScreenReplayBuffer,
    output_log: ScreenOutputLog,
    monitor_enabled: bool,
    activity_pending: bool,
    terminal: vt100::Parser,
}

impl ScreenWindowState {
    pub(super) fn new(index: usize) -> Self {
        Self {
            index,
            title: None,
            replay: ScreenReplayBuffer::default(),
            output_log: ScreenOutputLog::new(index),
            monitor_enabled: false,
            activity_pending: false,
            terminal: vt100::Parser::new(
                DEFAULT_TERMINAL_ROWS,
                DEFAULT_TERMINAL_COLS,
                0,
            ),
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

    pub(super) fn monitor_enabled(&self) -> bool {
        self.monitor_enabled
    }

    pub(super) fn set_monitor_enabled(&mut self, enabled: bool) {
        self.monitor_enabled = enabled;
        self.activity_pending = false;
    }

    pub(super) fn mark_activity(&mut self) -> bool {
        if !self.monitor_enabled || self.activity_pending {
            return false;
        }
        self.activity_pending = true;
        true
    }

    pub(super) fn clear_activity(&mut self) {
        self.activity_pending = false;
    }

    pub(super) fn terminal_screen(&self) -> &vt100::Screen {
        self.terminal.screen()
    }

    pub(super) fn resize_terminal(&mut self, rows: u16, cols: u16) {
        self.terminal
            .screen_mut()
            .set_size(rows.max(1), cols.max(1));
    }

    pub(super) fn replay_snapshot(&self) -> Vec<u8> {
        self.replay.snapshot()
    }

    pub(super) fn hardcopy_snapshot(
        &self,
        include_history: bool,
        rows: Option<u16>,
        cols: Option<u16>,
    ) -> Vec<u8> {
        if include_history {
            self.replay.snapshot()
        } else {
            self.replay.display_snapshot(rows, cols)
        }
    }

    pub(super) fn replay_len(&self) -> usize {
        self.replay.len()
    }

    pub(super) fn scrollback_lines(&self) -> usize {
        self.replay.scrollback_lines()
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

    pub(super) fn set_log_timestamp_enabled(&mut self, enabled: bool) {
        self.output_log.set_timestamp_enabled(enabled);
    }

    pub(super) fn toggle_log_timestamp_enabled(&mut self) {
        self.output_log.toggle_timestamp_enabled();
    }

    pub(super) fn set_log_timestamp_after(&mut self, seconds: u64) {
        self.output_log.set_timestamp_after(seconds);
    }

    pub(super) fn set_log_timestamp_format(&mut self, value: String) {
        self.output_log.set_timestamp_format(value);
    }

    pub(super) fn set_log_enabled(&mut self, enabled: bool) -> io::Result<()> {
        self.output_log.set_enabled(enabled)
    }

    pub(super) fn toggle_log_enabled(&mut self) -> io::Result<()> {
        self.output_log.toggle_enabled()
    }

    pub(super) fn append_replay(&mut self, bytes: &[u8], cols: Option<u16>) {
        self.terminal.process(bytes);
        self.replay.append(bytes, cols);
    }

    pub(super) fn append_output(&mut self, bytes: &[u8], cols: Option<u16>) {
        self.append_replay(bytes, cols);
        self.output_log.append(bytes, self.title.as_deref());
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