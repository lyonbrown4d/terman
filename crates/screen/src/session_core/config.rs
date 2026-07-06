use std::path::PathBuf;

use super::ScreenSessionBus;

impl ScreenSessionBus {
    pub(crate) fn set_hardcopy_dir(&self, path: PathBuf) {
        if let Ok(mut state) = self.inner.lock() {
            state.hardcopy_dir = Some(path);
        }
    }

    pub(crate) fn set_hardcopy_append(&self, append: bool) {
        if let Ok(mut state) = self.inner.lock() {
            state.hardcopy_append = append;
        }
    }

    pub(crate) fn set_buffer_file(&self, path: PathBuf) {
        if let Ok(mut state) = self.inner.lock() {
            state.buffer_file = path;
        }
    }

    pub(crate) fn set_scrollback_lines(&self, lines: usize) {
        if let Ok(mut state) = self.inner.lock() {
            let cols = state.cols;
            if let Some(window) = state.active_window_mut() {
                window.set_scrollback_lines(lines, cols);
            }
        }
    }

    pub(crate) fn set_window_title(&self, title: String) {
        if let Ok(mut state) = self.inner.lock() {
            if let Some(window) = state.active_window_mut() {
                window.set_title(title);
            }
        }
    }

    pub(crate) fn set_paste_buffer(&self, bytes: Vec<u8>) {
        if let Ok(mut state) = self.inner.lock() {
            state.paste_buffer = bytes;
        }
    }

    pub(crate) fn paste_buffer_snapshot(&self) -> Vec<u8> {
        self.inner
            .lock()
            .map(|state| state.paste_buffer.clone())
            .unwrap_or_default()
    }
}