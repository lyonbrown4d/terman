use std::sync::{Arc, Mutex, mpsc};

const DEFAULT_SCROLLBACK_LINES: usize = 1_000;
const DEFAULT_SCROLLBACK_COLS: usize = 80;
const MIN_SCROLLBACK_BYTES: usize = 4 * 1024;

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
    pub(crate) scrollback_lines: usize,
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
    replay: Vec<u8>,
    subscribers: Vec<ScreenSessionSubscriber>,
    attach_clients: usize,
    cols: Option<u16>,
    rows: Option<u16>,
    scrollback_lines: usize,
}

impl Default for ScreenSessionState {
    fn default() -> Self {
        Self {
            replay: Vec::new(),
            subscribers: Vec::new(),
            attach_clients: 0,
            cols: None,
            rows: None,
            scrollback_lines: DEFAULT_SCROLLBACK_LINES,
        }
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
            let replay = state.replay.clone();
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
                scrollback_lines: state.scrollback_lines,
            })
            .unwrap_or(ScreenSessionStatus {
                replay_bytes: 0,
                attach_clients: 0,
                cols: None,
                rows: None,
                scrollback_lines: DEFAULT_SCROLLBACK_LINES,
            })
    }

    pub(crate) fn clear_replay(&self) {
        if let Ok(mut state) = self.inner.lock() {
            state.replay.clear();
        }
    }

    pub(crate) fn set_scrollback_lines(&self, lines: usize) {
        if let Ok(mut state) = self.inner.lock() {
            state.scrollback_lines = lines;
            trim_replay(&mut state);
        }
    }

    pub(crate) fn publish_transient_output(&self, bytes: &[u8]) {
        self.publish(ScreenSessionEvent::Output(bytes.to_vec()), |_| {});
    }

    pub(crate) fn publish_output(&self, bytes: &[u8]) {
        self.publish(ScreenSessionEvent::Output(bytes.to_vec()), |state| {
            state.replay.extend_from_slice(bytes);
            trim_replay(state);
        });
    }

    pub(crate) fn publish_resize(&self, cols: u16, rows: u16) {
        self.publish(ScreenSessionEvent::Resize { cols, rows }, |state| {
            state.cols = Some(cols);
            state.rows = Some(rows);
            trim_replay(state);
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

fn trim_replay(state: &mut ScreenSessionState) {
    let max_bytes = max_replay_bytes(state.scrollback_lines, state.cols);
    if state.replay.len() > max_bytes {
        let overflow = state.replay.len() - max_bytes;
        state.replay.drain(..overflow);
    }
}

fn max_replay_bytes(scrollback_lines: usize, cols: Option<u16>) -> usize {
    if scrollback_lines == 0 {
        return 0;
    }
    let cols = cols.map(usize::from).unwrap_or(DEFAULT_SCROLLBACK_COLS);
    scrollback_lines
        .saturating_mul(cols)
        .max(MIN_SCROLLBACK_BYTES)
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
        let (replay, subscription) = bus.subscribe_with_replay(None);
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
        let (_replay, subscription) = bus.subscribe_with_replay(None);

        assert_eq!(bus.status_snapshot().attach_clients, 1);
        drop(subscription);
        assert_eq!(bus.status_snapshot().attach_clients, 0);
    }

    #[test]
    fn detaches_one_client_without_broadcasting() {
        let bus = ScreenSessionBus::new();
        let (_replay, subscription) = bus.subscribe_with_replay(Some(String::from("client")));

        bus.detach_client("client");

        assert!(subscription.recv().is_err());
        drop(subscription);
        assert_eq!(bus.status_snapshot().attach_clients, 0);
    }

    #[test]
    fn updates_scrollback_limit() {
        let bus = ScreenSessionBus::new();
        bus.set_scrollback_lines(0);
        bus.publish_output(b"hello");

        assert_eq!(bus.status_snapshot().scrollback_lines, 0);
        assert!(bus.replay_snapshot().is_empty());
    }

    #[test]
    fn models_attach_control_events() {
        assert_eq!(
            ScreenControlEvent::Input(b"x".to_vec()),
            ScreenControlEvent::Input(b"x".to_vec())
        );
    }
}