use std::sync::{Arc, Mutex, mpsc};

mod events;
mod logging;
mod log_control;
mod replay;
mod registers; mod state;
mod status;
mod window;

pub(crate) use events::{ScreenControlEvent, ScreenSessionEvent};
pub(crate) use status::{ScreenSessionStatus, ScreenWindowStatus};
use state::{ScreenRemovedWindow, ScreenSessionState, ScreenSessionSubscriber, fallback_status, session_status};
use window::ScreenWindowState;

#[derive(Clone, Default)]
pub(crate) struct ScreenSessionBus {
    inner: Arc<Mutex<ScreenSessionState>>,
}

pub(crate) struct ScreenSessionSubscription {
    receiver: mpsc::Receiver<ScreenSessionEvent>,
    bus: ScreenSessionBus,
    active: bool,
}

impl ScreenSessionSubscription {
    pub(crate) fn recv(&self) -> Result<ScreenSessionEvent, mpsc::RecvError> {
        self.receiver.recv()
    }

    #[cfg(test)]
    pub(crate) fn try_recv(&self) -> Result<ScreenSessionEvent, mpsc::TryRecvError> {
        self.receiver.try_recv()
    }
}

impl Drop for ScreenSessionSubscription {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let Ok(mut state) = self.bus.inner.lock() else {
            return;
        };
        state.attach_clients = state.attach_clients.saturating_sub(1);
    }
}

impl ScreenSessionBus {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn subscribe_with_replay(
        &self,
        client_id: Option<String>,
    ) -> (Vec<u8>, ScreenSessionSubscription) {
        let (tx, rx) = mpsc::channel();
        let mut active = false;
        let replay = if let Ok(mut state) = self.inner.lock() {
            let replay = state
                .active_window()
                .map(ScreenWindowState::replay_snapshot)
                .unwrap_or_default();
            state.subscribers.push(ScreenSessionSubscriber {
                client_id,
                sender: tx,
            });
            state.attach_clients += 1;
            active = true;
            replay
        } else {
            Vec::new()
        };

        (
            replay,
            ScreenSessionSubscription {
                receiver: rx,
                bus: self.clone(),
                active,
            },
        )
    }

    #[cfg(test)]
    pub(crate) fn replay_snapshot(&self) -> Vec<u8> {
        self.hardcopy_snapshot(true)
    }

    pub(crate) fn hardcopy_snapshot(&self, include_history: bool) -> Vec<u8> {
        self.inner
            .lock()
            .ok()
            .and_then(|state| {
                let rows = state.rows;
                let cols = state.cols;
                state
                    .active_window()
                    .map(|window| window.hardcopy_snapshot(include_history, rows, cols))
            })
            .unwrap_or_default()
    }

    pub(crate) fn status_snapshot(&self) -> ScreenSessionStatus {
        self.inner
            .lock()
            .map(|state| session_status(&state))
            .unwrap_or_else(|_| fallback_status())
    }

    pub(crate) fn set_hardcopy_dir(&self, path: std::path::PathBuf) {
        if let Ok(mut state) = self.inner.lock() {
            state.hardcopy_dir = Some(path);
        }
    }

    pub(crate) fn set_hardcopy_append(&self, append: bool) {
        if let Ok(mut state) = self.inner.lock() {
            state.hardcopy_append = append;
        }
    }

    pub(crate) fn set_buffer_file(&self, path: std::path::PathBuf) {
        if let Ok(mut state) = self.inner.lock() {
            state.buffer_file = path;
        }
    }

    #[cfg(test)]
    pub(crate) fn add_window(&self, index: usize, title: Option<String>) {
        self.add_window_with_scrollback(index, title, replay::DEFAULT_SCROLLBACK_LINES);
    }

    pub(crate) fn add_window_with_scrollback(
        &self,
        index: usize,
        title: Option<String>,
        scrollback_lines: usize,
    ) {
        if let Ok(mut state) = self.inner.lock() {
            state.add_window(index, title, scrollback_lines);
        }
    }

    pub(crate) fn select_window(&self, index: usize) -> Option<Vec<u8>> {
        self.inner
            .lock()
            .ok()
            .and_then(|mut state| state.select_window(index))
    }

    pub(crate) fn select_last_window(&self) -> Option<Vec<u8>> {
        self.inner
            .lock()
            .ok()
            .and_then(|mut state| state.select_last_window())
    }
    pub(crate) fn renumber_window(&self, source: usize, index: usize) {
        if let Ok(mut state) = self.inner.lock() {
            state.renumber_window(source, index);
        }
    }
    pub(crate) fn remove_window(&self, index: usize) -> Option<ScreenRemovedWindow> {
        self.inner
            .lock()
            .ok()
            .and_then(|mut state| state.remove_window(index))
    }
    pub(crate) fn clear_replay(&self) {
        if let Ok(mut state) = self.inner.lock() {
            if let Some(window) = state.active_window_mut() {
                window.clear_replay();
            }
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

    pub(crate) fn publish_transient_output(&self, bytes: &[u8]) {
        self.broadcast(ScreenSessionEvent::Output(bytes.to_vec()));
    }
    pub(crate) fn last_message_snapshot(&self) -> Vec<u8> {
        self.inner
            .lock()
            .map(|state| state.last_message.clone())
            .unwrap_or_default()
    }

    pub(crate) fn publish_message(&self, bytes: &[u8]) {
        let message = bytes.to_vec();
        let event = ScreenSessionEvent::Output(message.clone());
        self.publish(event, |state| {
            state.last_message = message;
        });
    }

    #[cfg(test)]
    pub(crate) fn publish_output(&self, bytes: &[u8]) {
        let active_window = self.inner.lock().map(|state| state.active_window).ok();
        if let Some(active_window) = active_window {
            self.publish_window_output(active_window, bytes);
        }
    }

    pub(crate) fn publish_window_output(&self, index: usize, bytes: &[u8]) {
        let Ok(mut state) = self.inner.lock() else {
            return;
        };
        let cols = state.cols;
        let active = state.active_window == index;
        if let Some(window) = state.window_mut(index) {
            window.append_output(bytes, cols);
        }
        if active {
            state.subscribers.retain(|subscriber| {
                subscriber
                    .sender
                    .send(ScreenSessionEvent::Output(bytes.to_vec()))
                    .is_ok()
            });
        }
    }

    pub(crate) fn publish_resize(&self, cols: u16, rows: u16) {
        self.publish(ScreenSessionEvent::Resize { cols, rows }, |state| {
            state.cols = Some(cols);
            state.rows = Some(rows);
            for window in &mut state.windows {
                window.trim_to_cols(state.cols);
            }
        });
    }

    pub(crate) fn detach_client(&self, client_id: &str) {
        if let Ok(mut state) = self.inner.lock() {
            state
                .subscribers
                .retain(|subscriber| subscriber.client_id.as_deref() != Some(client_id));
        }
    }

    pub(crate) fn publish_detach(&self) {
        self.broadcast(ScreenSessionEvent::Detach);
    }

    pub(crate) fn publish_exit(&self, code: i32) {
        self.broadcast(ScreenSessionEvent::Exit(code));
    }

    fn publish(&self, event: ScreenSessionEvent, update: impl FnOnce(&mut ScreenSessionState)) {
        let Ok(mut state) = self.inner.lock() else {
            return;
        };
        update(&mut state);
        state
            .subscribers
            .retain(|subscriber| subscriber.sender.send(event.clone()).is_ok());
    }

    fn broadcast(&self, event: ScreenSessionEvent) {
        self.publish(event, |_| {});
    }
}

#[cfg(test)]
#[path = "session_core/tests.rs"]
mod tests;