#![allow(dead_code)]

use std::sync::{Arc, Mutex, mpsc};

pub(crate) use crate::session_model::{TmuxControlEvent, TmuxSessionEvent, TmuxSessionStatus};
use crate::session_state::{
    TmuxSessionState, TmuxSessionSubscriber, select_window_state, send_to_subscribers,
};

#[derive(Clone)]
pub(crate) struct TmuxSessionBus {
    inner: Arc<Mutex<TmuxSessionState>>,
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
            state.window_capture(index).map(ToOwned::to_owned)
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
        state.clear_window_capture(index)
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

    pub(crate) fn publish_window_frame(&self, index: u32, frame: Vec<u8>, capture: Vec<u8>) {
        let Ok(mut state) = self.inner.lock() else { return; };
        state.replace_window_output(index, frame.clone(), capture);
        if index == state.active_window {
            send_to_subscribers(&mut state, TmuxSessionEvent::Output(frame));
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

#[cfg(test)]
#[path = "session_core_tests.rs"]
mod tests;