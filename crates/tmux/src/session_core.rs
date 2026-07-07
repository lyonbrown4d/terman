#![allow(dead_code)]

use std::sync::{Arc, Mutex, mpsc};

const MAX_REPLAY_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TmuxSessionEvent {
    Output(Vec<u8>),
    Resize { cols: u16, rows: u16 },
    Detach,
    Exit(i32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TmuxControlEvent {
    Input(Vec<u8>),
    Resize { cols: u16, rows: u16 },
    NewWindow { index: u32, name: String, command: Option<String> },
    RenameWindow { index: u32, name: String },
    KillWindow { index: u32 },
    SelectWindow { index: u32 },
    Terminate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TmuxSessionStatus {
    pub(crate) replay_bytes: usize,
    pub(crate) attached_clients: u32,
    pub(crate) windows: u32,
    pub(crate) active_window: u32,
    pub(crate) window_indexes: Vec<u32>,
    pub(crate) window_names: Vec<String>,
    pub(crate) cols: Option<u16>,
    pub(crate) rows: Option<u16>,
}

#[derive(Clone)]
pub(crate) struct TmuxSessionBus {
    inner: Arc<Mutex<TmuxSessionState>>,
}

struct TmuxSessionSubscriber {
    client_id: Option<String>,
    sender: mpsc::Sender<TmuxSessionEvent>,
}

struct TmuxWindowReplay {
    index: u32,
    name: String,
    replay: Vec<u8>,
}

struct TmuxSessionState {
    windows: Vec<TmuxWindowReplay>,
    subscribers: Vec<TmuxSessionSubscriber>,
    attached_clients: u32,
    active_window: u32,
    last_window: Option<u32>,
    cols: Option<u16>,
    rows: Option<u16>,
}

pub(crate) struct TmuxSessionSubscription {
    receiver: mpsc::Receiver<TmuxSessionEvent>,
    bus: TmuxSessionBus,
    active: bool,
}

impl TmuxSessionSubscription {
    pub(crate) fn recv(&self) -> Result<TmuxSessionEvent, mpsc::RecvError> {
        self.receiver.recv()
    }

    #[cfg(test)]
    pub(crate) fn try_recv(&self) -> Result<TmuxSessionEvent, mpsc::TryRecvError> {
        self.receiver.try_recv()
    }
}

impl Drop for TmuxSessionSubscription {
    fn drop(&mut self) {
        if !self.active { return; }
        let Ok(mut state) = self.bus.inner.lock() else { return; };
        state.attached_clients = state.attached_clients.saturating_sub(1);
    }
}

impl TmuxSessionBus {
    pub(crate) fn new(windows: u32) -> Self {
        Self { inner: Arc::new(Mutex::new(TmuxSessionState::new(windows))) }
    }

    pub(crate) fn subscribe_with_replay(&self, client_id: Option<String>) -> (Vec<u8>, TmuxSessionSubscription) {
        let (tx, rx) = mpsc::channel();
        let mut active = false;
        let replay = if let Ok(mut state) = self.inner.lock() {
            let replay = state.active_replay().to_vec();
            state.subscribers.push(TmuxSessionSubscriber { client_id, sender: tx });
            state.attached_clients = state.attached_clients.saturating_add(1);
            active = true;
            replay
        } else { Vec::new() };
        (replay, TmuxSessionSubscription { receiver: rx, bus: self.clone(), active })
    }

    pub(crate) fn replay_snapshot(&self) -> Vec<u8> {
        self.inner.lock().map(|state| state.active_replay().to_vec()).unwrap_or_default()
    }

    pub(crate) fn window_replay_snapshot(&self, index: Option<u32>) -> Option<Vec<u8>> {
        self.inner.lock().ok().and_then(|state| {
            let index = index.unwrap_or(state.active_window);
            state
                .windows
                .iter()
                .find(|window| window.index == index)
                .map(|window| window.replay.clone())
        })
    }
    pub(crate) fn status_snapshot(&self) -> TmuxSessionStatus {
        self.inner.lock().map(|state| TmuxSessionStatus {
            replay_bytes: state.active_replay().len(),
            attached_clients: state.attached_clients,
            windows: state.windows.len().max(1) as u32,
            active_window: state.active_window,
            window_indexes: state.windows.iter().map(|window| window.index).collect(),
            window_names: state.windows.iter().map(|window| window.name.clone()).collect(),
            cols: state.cols,
            rows: state.rows,
        }).unwrap_or(TmuxSessionStatus {
            replay_bytes: 0,
            attached_clients: 0,
            windows: 1,
            active_window: 0,
            window_indexes: vec![0],
            window_names: vec![String::from("0")],
            cols: None,
            rows: None,
        })
    }

    pub(crate) fn clear_window_replay(&self, index: Option<u32>) -> bool {
        let Ok(mut state) = self.inner.lock() else { return false; };
        let index = index.unwrap_or(state.active_window);
        let Some(window) = state.windows.iter_mut().find(|window| window.index == index) else { return false; };
        window.replay.clear();
        true
    }
    pub(crate) fn clear_replay(&self) {
        if let Ok(mut state) = self.inner.lock() { state.active_replay_mut().clear(); }
    }

    pub(crate) fn publish_transient_output(&self, bytes: &[u8]) {
        self.publish(TmuxSessionEvent::Output(bytes.to_vec()), |_| {});
    }

    pub(crate) fn publish_output(&self, bytes: &[u8]) {
        let active = self.status_snapshot().active_window;
        self.publish_window_output(active, bytes);
    }

    pub(crate) fn publish_window_output(&self, index: u32, bytes: &[u8]) {
        let Ok(mut state) = self.inner.lock() else { return; };
        state.append_window_output(index, bytes);
        if index == state.active_window {
            send_to_subscribers(&mut state, TmuxSessionEvent::Output(bytes.to_vec()));
        }
    }

    pub(crate) fn publish_resize(&self, cols: u16, rows: u16) {
        self.publish(TmuxSessionEvent::Resize { cols, rows }, |state| { state.cols = Some(cols); state.rows = Some(rows); });
    }

    pub(crate) fn set_windows(&self, windows: u32) {
        if let Ok(mut state) = self.inner.lock() { state.set_window_count(windows.max(1)); }
    }

    pub(crate) fn add_window(&self, index: u32, name: String) {
        if let Ok(mut state) = self.inner.lock() { state.ensure_window(index, name); }
    }

    pub(crate) fn rename_window(&self, index: u32, name: String) -> bool {
        let Ok(mut state) = self.inner.lock() else { return false; };
        let Some(window) = state.windows.iter_mut().find(|window| window.index == index) else { return false; };
        window.name = name;
        true
    }

    pub(crate) fn remove_window(&self, index: u32) {
        if let Ok(mut state) = self.inner.lock() { state.remove_window(index); }
    }

    pub(crate) fn select_window(&self, index: u32) -> bool {
        let Ok(mut state) = self.inner.lock() else { return false; };
        if !state.has_window(index) { return false; }
        select_window_state(&mut state, index);
        let replay = state.active_replay().to_vec();
        send_to_subscribers(&mut state, TmuxSessionEvent::Output(b"\x1bc".to_vec()));
        if !replay.is_empty() { send_to_subscribers(&mut state, TmuxSessionEvent::Output(replay)); }
        true
    }

    pub(crate) fn select_last_window(&self) -> Option<u32> {
        let mut state = self.inner.lock().ok()?;
        let index = state.last_window?;
        if !state.has_window(index) {
            state.last_window = None;
            return None;
        }
        select_window_state(&mut state, index);
        Some(index)
    }
    pub(crate) fn detach_client(&self, client_id: &str) {
        if let Ok(mut state) = self.inner.lock() {
            state.subscribers.retain(|subscriber| subscriber.client_id.as_deref() != Some(client_id));
        }
    }

    pub(crate) fn publish_detach(&self) { self.publish(TmuxSessionEvent::Detach, |_| {}); }
    pub(crate) fn publish_exit(&self, code: i32) { self.publish(TmuxSessionEvent::Exit(code), |_| {}); }

    fn publish(&self, event: TmuxSessionEvent, update: impl FnOnce(&mut TmuxSessionState)) {
        let Ok(mut state) = self.inner.lock() else { return; };
        update(&mut state);
        send_to_subscribers(&mut state, event);
    }
}

impl TmuxSessionState {
    fn new(windows: u32) -> Self {
        let mut state = Self { windows: Vec::new(), subscribers: Vec::new(), attached_clients: 0, active_window: 0, last_window: None, cols: None, rows: None };
        state.set_window_count(windows.max(1));
        state
    }

    fn active_replay(&self) -> &[u8] {
        self.windows.iter().find(|window| window.index == self.active_window).map(|window| window.replay.as_slice()).unwrap_or(&[])
    }

    fn active_replay_mut(&mut self) -> &mut Vec<u8> {
        self.ensure_window(self.active_window, self.active_window.to_string());
        self.windows.iter_mut().find(|window| window.index == self.active_window).map(|window| &mut window.replay).expect("active window exists")
    }

    fn has_window(&self, index: u32) -> bool {
        self.windows.iter().any(|window| window.index == index)
    }

    fn ensure_window(&mut self, index: u32, name: String) {
        if !self.has_window(index) {
            self.windows.push(TmuxWindowReplay { index, name, replay: Vec::new() });
            self.windows.sort_by_key(|window| window.index);
        }
    }

    fn append_window_output(&mut self, index: u32, bytes: &[u8]) {
        self.ensure_window(index, index.to_string());
        let replay = self.windows.iter_mut().find(|window| window.index == index).expect("window exists");
        replay.replay.extend_from_slice(bytes);
        if replay.replay.len() > MAX_REPLAY_BYTES {
            let overflow = replay.replay.len() - MAX_REPLAY_BYTES;
            replay.replay.drain(..overflow);
        }
    }

    fn set_window_count(&mut self, windows: u32) {
        for index in 0..windows { self.ensure_window(index, index.to_string()); }
        self.windows.retain(|window| window.index < windows);
        if !self.has_window(self.active_window) {
            self.active_window = self.windows.first().map(|window| window.index).unwrap_or(0);
        }
    }

    fn remove_window(&mut self, index: u32) {
        self.windows.retain(|window| window.index != index);
        if self.last_window == Some(index) { self.last_window = None; }
        if self.windows.is_empty() { self.ensure_window(0, String::from("0")); }
        if !self.has_window(self.active_window) {
            self.active_window = self.windows.first().map(|window| window.index).unwrap_or(0);
        }
    }
}

fn select_window_state(state: &mut TmuxSessionState, index: u32) {
    if state.active_window != index {
        state.last_window = Some(state.active_window);
    }
    state.active_window = index;
}
fn send_to_subscribers(state: &mut TmuxSessionState, event: TmuxSessionEvent) {
    state.subscribers.retain(|subscriber| subscriber.sender.send(event.clone()).is_ok());
}

#[cfg(test)]
#[path = "session_core_tests.rs"]
mod tests;