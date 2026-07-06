use std::{
    io,
    sync::{Arc, Mutex, mpsc},
};

mod logging;
mod replay;
mod window;

use replay::DEFAULT_SCROLLBACK_LINES;
use window::ScreenWindowState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ScreenSessionEvent {
    Output(Vec<u8>),
    Resize { cols: u16, rows: u16 },
    Detach,
    Exit(i32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ScreenControlEvent {
    Input(Vec<u8>),
    Resize { cols: u16, rows: u16 },
    Terminate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScreenWindowStatus {
    pub(crate) index: usize,
    pub(crate) title: Option<String>,
    pub(crate) active: bool,
    pub(crate) replay_bytes: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScreenSessionStatus {
    pub(crate) replay_bytes: usize,
    pub(crate) attach_clients: usize,
    pub(crate) cols: Option<u16>,
    pub(crate) rows: Option<u16>,
    pub(crate) scrollback_lines: usize,
    pub(crate) window_title: Option<String>,
    pub(crate) active_window: usize,
    pub(crate) windows: Vec<ScreenWindowStatus>,
}

#[derive(Clone, Default)]
pub(crate) struct ScreenSessionBus {
    inner: Arc<Mutex<ScreenSessionState>>,
}

struct ScreenSessionSubscriber {
    client_id: Option<String>,
    sender: mpsc::Sender<ScreenSessionEvent>,
}

struct ScreenSessionState {
    windows: Vec<ScreenWindowState>,
    active_window: usize,
    paste_buffer: Vec<u8>,
    subscribers: Vec<ScreenSessionSubscriber>,
    attach_clients: usize,
    cols: Option<u16>,
    rows: Option<u16>,
}

impl Default for ScreenSessionState {
    fn default() -> Self {
        Self {
            windows: vec![ScreenWindowState::new(0)],
            active_window: 0,
            paste_buffer: Vec::new(),
            subscribers: Vec::new(),
            attach_clients: 0,
            cols: None,
            rows: None,
        }
    }
}

impl ScreenSessionState {
    fn active_window(&self) -> Option<&ScreenWindowState> {
        self.windows.get(self.active_window)
    }

    fn active_window_mut(&mut self) -> Option<&mut ScreenWindowState> {
        self.windows.get_mut(self.active_window)
    }
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

    pub(crate) fn replay_snapshot(&self) -> Vec<u8> {
        self.inner
            .lock()
            .ok()
            .and_then(|state| state.active_window().map(ScreenWindowState::replay_snapshot))
            .unwrap_or_default()
    }

    pub(crate) fn status_snapshot(&self) -> ScreenSessionStatus {
        self.inner
            .lock()
            .map(|state| session_status(&state))
            .unwrap_or_else(|_| fallback_status())
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

    pub(crate) fn publish_transient_output(&self, bytes: &[u8]) {
        self.publish(ScreenSessionEvent::Output(bytes.to_vec()), |_| {});
    }

    pub(crate) fn publish_output(&self, bytes: &[u8]) {
        self.publish(ScreenSessionEvent::Output(bytes.to_vec()), |state| {
            let cols = state.cols;
            if let Some(window) = state.active_window_mut() {
                window.append_output(bytes, cols);
            }
        });
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
        self.publish(ScreenSessionEvent::Detach, |_| {});
    }

    pub(crate) fn publish_exit(&self, code: i32) {
        self.publish(ScreenSessionEvent::Exit(code), |_| {});
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
}

fn session_status(state: &ScreenSessionState) -> ScreenSessionStatus {
    let active = state.active_window();
    let replay_bytes = active.map(ScreenWindowState::replay_len).unwrap_or_default();
    let window_title = active.and_then(ScreenWindowState::title).map(str::to_string);
    let scrollback_lines = active
        .map(ScreenWindowState::scrollback_lines)
        .unwrap_or(DEFAULT_SCROLLBACK_LINES);
    ScreenSessionStatus {
        replay_bytes,
        attach_clients: state.attach_clients,
        cols: state.cols,
        rows: state.rows,
        scrollback_lines,
        window_title,
        active_window: state.active_window,
        windows: state
            .windows
            .iter()
            .enumerate()
            .map(|(index, window)| window.status(index == state.active_window))
            .collect(),
    }
}

fn fallback_status() -> ScreenSessionStatus {
    ScreenSessionStatus {
        replay_bytes: 0,
        attach_clients: 0,
        cols: None,
        rows: None,
        scrollback_lines: DEFAULT_SCROLLBACK_LINES,
        window_title: None,
        active_window: 0,
        windows: vec![ScreenWindowStatus {
            index: 0,
            title: None,
            active: true,
            replay_bytes: 0,
        }],
    }
}

#[cfg(test)]
#[path = "session_core/tests.rs"]
mod tests;