use std::sync::{Arc, Mutex, mpsc};

use crate::region_types::BLANK_SCREEN_WINDOW_INDEX;

mod config;
mod events;
mod logging;
mod monitor;
mod silence;
mod log_control;
mod region_bus;
mod region_layout;
mod region_render;
mod replay;
mod registers; mod state;
mod status;
mod window;
mod wrap;

pub(crate) use events::{ScreenControlEvent, ScreenSessionEvent};
pub(crate) use silence::DEFAULT_SILENCE_SECONDS;
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
            let replay = if state.has_multiple_regions()
                || state.active_window == BLANK_SCREEN_WINDOW_INDEX
            {
                state.render_regions()
            } else {
                state
                    .active_window()
                    .map(ScreenWindowState::attach_replay)
                    .unwrap_or_default()
            };
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
    pub(crate) fn publish_display_control(&self, bytes: &[u8]) {
        let Ok(mut state) = self.inner.lock() else { return; };
        let cols = state.cols;
        let active_window = state.active_window;
        let Some(window) = state.window_mut(active_window) else { return; };
        window.append_replay(bytes, cols);
        let event = ScreenSessionEvent::Output(bytes.to_vec());
        state.subscribers.retain(|subscriber| subscriber.sender.send(event.clone()).is_ok());
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

    pub(crate) fn publish_resize(&self, cols: u16, rows: u16) {
        self.publish(ScreenSessionEvent::Resize { cols, rows }, |state| {
            state.cols = Some(cols);
            state.rows = Some(rows);
            state.resize_terminals(rows, cols);
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
