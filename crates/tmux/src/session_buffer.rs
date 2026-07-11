use crate::session_core::TmuxSessionBus;

impl TmuxSessionBus {
    pub(crate) fn set_buffer(&self, bytes: Vec<u8>) {
        if let Ok(mut state) = self.inner.lock() {
            state.buffer = bytes;
        }
    }

    pub(crate) fn buffer_snapshot(&self) -> Vec<u8> {
        self.inner
            .lock()
            .map(|state| state.buffer.clone())
            .unwrap_or_default()
    }
}