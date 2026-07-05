use std::sync::{Arc, Mutex, mpsc};

const MAX_REPLAY_BYTES: usize = 64 * 1024;

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
pub(crate) struct ScreenSessionStatus {
    pub(crate) replay_bytes: usize,
    pub(crate) attach_clients: usize,
    pub(crate) cols: Option<u16>,
    pub(crate) rows: Option<u16>,
}

#[derive(Clone, Default)]
pub(crate) struct ScreenSessionBus {
    inner: Arc<Mutex<ScreenSessionState>>,
}

#[derive(Default)]
struct ScreenSessionState {
    replay: Vec<u8>,
    subscribers: Vec<mpsc::Sender<ScreenSessionEvent>>,
    attach_clients: usize,
    cols: Option<u16>,
    rows: Option<u16>,
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

    pub(crate) fn subscribe_with_replay(&self) -> (Vec<u8>, ScreenSessionSubscription) {
        let (tx, rx) = mpsc::channel();
        let mut active = false;
        let replay = if let Ok(mut state) = self.inner.lock() {
            let replay = state.replay.clone();
            state.subscribers.push(tx);
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
            .map(|state| state.replay.clone())
            .unwrap_or_default()
    }

    pub(crate) fn status_snapshot(&self) -> ScreenSessionStatus {
        self.inner
            .lock()
            .map(|state| ScreenSessionStatus {
                replay_bytes: state.replay.len(),
                attach_clients: state.attach_clients,
                cols: state.cols,
                rows: state.rows,
            })
            .unwrap_or(ScreenSessionStatus {
                replay_bytes: 0,
                attach_clients: 0,
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
        self.publish(ScreenSessionEvent::Output(bytes.to_vec()), |_| {});
    }

    pub(crate) fn publish_output(&self, bytes: &[u8]) {
        self.publish(ScreenSessionEvent::Output(bytes.to_vec()), |state| {
            state.replay.extend_from_slice(bytes);
            if state.replay.len() > MAX_REPLAY_BYTES {
                let overflow = state.replay.len() - MAX_REPLAY_BYTES;
                state.replay.drain(..overflow);
            }
        });
    }

    pub(crate) fn publish_resize(&self, cols: u16, rows: u16) {
        self.publish(ScreenSessionEvent::Resize { cols, rows }, |state| {
            state.cols = Some(cols);
            state.rows = Some(rows);
        });
    }

    pub(crate) fn detach_client(&self) {
        if let Ok(mut state) = self.inner.lock() {
            state.attach_clients = state.attach_clients.saturating_sub(1);
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
            .retain(|subscriber| subscriber.send(event.clone()).is_ok());
    }
}

#[cfg(test)]
mod tests {
    use super::{ScreenControlEvent, ScreenSessionBus, ScreenSessionEvent};

    #[test]
    fn replays_recent_output_to_attach_clients() {
        let bus = ScreenSessionBus::new();
        bus.publish_output(b"hello");

        assert_eq!(bus.replay_snapshot(), b"hello".to_vec());
    }

    #[test]
    fn subscribes_with_replay_without_losing_snapshot() {
        let bus = ScreenSessionBus::new();
        bus.publish_output(b"hello");
        let (replay, subscription) = bus.subscribe_with_replay();
        bus.publish_output(b"!");

        assert_eq!(replay, b"hello".to_vec());
        assert_eq!(
            subscription.try_recv(),
            Ok(ScreenSessionEvent::Output(b"!".to_vec()))
        );
    }

    #[test]
    fn tracks_attach_client_count_for_replay_subscriptions() {
        let bus = ScreenSessionBus::new();
        let (_replay, subscription) = bus.subscribe_with_replay();

        assert_eq!(bus.status_snapshot().attach_clients, 1);
        drop(subscription);
        assert_eq!(bus.status_snapshot().attach_clients, 0);
    }

    #[test]
    fn detaches_one_client_without_broadcasting() {
        let bus = ScreenSessionBus::new();
        let (_replay, subscription) = bus.subscribe_with_replay();

        bus.detach_client();

        assert_eq!(bus.status_snapshot().attach_clients, 0);
        assert!(subscription.try_recv().is_err());
    }

    #[test]
    fn models_attach_control_events() {
        assert_eq!(
            ScreenControlEvent::Input(b"x".to_vec()),
            ScreenControlEvent::Input(b"x".to_vec())
        );
    }
}



