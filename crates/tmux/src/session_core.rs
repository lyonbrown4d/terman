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
    Terminate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TmuxSessionStatus {
    pub(crate) replay_bytes: usize,
    pub(crate) attached_clients: u32,
    pub(crate) windows: u32,
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

struct TmuxSessionState {
    replay: Vec<u8>,
    subscribers: Vec<TmuxSessionSubscriber>,
    attached_clients: u32,
    windows: u32,
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
        if !self.active {
            return;
        }
        let Ok(mut state) = self.bus.inner.lock() else {
            return;
        };
        state.attached_clients = state.attached_clients.saturating_sub(1);
    }
}

impl TmuxSessionBus {
    pub(crate) fn new(windows: u32) -> Self {
        Self {
            inner: Arc::new(Mutex::new(TmuxSessionState {
                replay: Vec::new(),
                subscribers: Vec::new(),
                attached_clients: 0,
                windows: windows.max(1),
                cols: None,
                rows: None,
            })),
        }
    }

    pub(crate) fn subscribe_with_replay(
        &self,
        client_id: Option<String>,
    ) -> (Vec<u8>, TmuxSessionSubscription) {
        let (tx, rx) = mpsc::channel();
        let mut active = false;
        let replay = if let Ok(mut state) = self.inner.lock() {
            let replay = state.replay.clone();
            state.subscribers.push(TmuxSessionSubscriber {
                client_id,
                sender: tx,
            });
            state.attached_clients = state.attached_clients.saturating_add(1);
            active = true;
            replay
        } else {
            Vec::new()
        };

        (
            replay,
            TmuxSessionSubscription {
                receiver: rx,
                bus: self.clone(),
                active,
            },
        )
    }

    pub(crate) fn replay_snapshot(&self) -> Vec<u8> {
        self.inner
            .lock()
            .map(|state| state.replay.clone())
            .unwrap_or_default()
    }

    pub(crate) fn status_snapshot(&self) -> TmuxSessionStatus {
        self.inner
            .lock()
            .map(|state| TmuxSessionStatus {
                replay_bytes: state.replay.len(),
                attached_clients: state.attached_clients,
                windows: state.windows,
                cols: state.cols,
                rows: state.rows,
            })
            .unwrap_or(TmuxSessionStatus {
                replay_bytes: 0,
                attached_clients: 0,
                windows: 1,
                cols: None,
                rows: None,
            })
    }

    pub(crate) fn clear_replay(&self) {
        if let Ok(mut state) = self.inner.lock() {
            state.replay.clear();
        }
    }

    pub(crate) fn publish_transient_output(&self, bytes: &[u8]) {
        self.publish(TmuxSessionEvent::Output(bytes.to_vec()), |_| {});
    }

    pub(crate) fn publish_output(&self, bytes: &[u8]) {
        self.publish(TmuxSessionEvent::Output(bytes.to_vec()), |state| {
            state.replay.extend_from_slice(bytes);
            if state.replay.len() > MAX_REPLAY_BYTES {
                let overflow = state.replay.len() - MAX_REPLAY_BYTES;
                state.replay.drain(..overflow);
            }
        });
    }

    pub(crate) fn publish_resize(&self, cols: u16, rows: u16) {
        self.publish(TmuxSessionEvent::Resize { cols, rows }, |state| {
            state.cols = Some(cols);
            state.rows = Some(rows);
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
        self.publish(TmuxSessionEvent::Detach, |_| {});
    }

    pub(crate) fn publish_exit(&self, code: i32) {
        self.publish(TmuxSessionEvent::Exit(code), |_| {});
    }

    fn publish(&self, event: TmuxSessionEvent, update: impl FnOnce(&mut TmuxSessionState)) {
        let Ok(mut state) = self.inner.lock() else {
            return;
        };
        update(&mut state);
        state
            .subscribers
            .retain(|subscriber| subscriber.sender.send(event.clone()).is_ok());
    }
}

#[cfg(test)]
mod tests {
    use super::{TmuxSessionBus, TmuxSessionEvent};

    #[test]
    fn replays_recent_output_to_attach_clients() {
        let bus = TmuxSessionBus::new(1);
        bus.publish_output(b"hello");

        assert_eq!(bus.replay_snapshot(), b"hello".to_vec());
    }

    #[test]
    fn subscribes_with_replay_without_losing_snapshot() {
        let bus = TmuxSessionBus::new(1);
        bus.publish_output(b"hello");
        let (replay, subscription) = bus.subscribe_with_replay(None);
        bus.publish_output(b"!");

        assert_eq!(replay, b"hello".to_vec());
        assert_eq!(
            subscription.try_recv(),
            Ok(TmuxSessionEvent::Output(b"!".to_vec()))
        );
    }

    #[test]
    fn tracks_attach_client_count() {
        let bus = TmuxSessionBus::new(2);
        let (_replay, subscription) = bus.subscribe_with_replay(None);

        assert_eq!(bus.status_snapshot().attached_clients, 1);
        assert_eq!(bus.status_snapshot().windows, 2);
        drop(subscription);
        assert_eq!(bus.status_snapshot().attached_clients, 0);
    }

    #[test]
    fn detaches_one_attach_client() {
        let bus = TmuxSessionBus::new(1);
        let (_replay, subscription) = bus.subscribe_with_replay(Some(String::from("client")));

        bus.detach_client("client");

        assert!(subscription.recv().is_err());
        drop(subscription);
        assert_eq!(bus.status_snapshot().attached_clients, 0);
    }
}