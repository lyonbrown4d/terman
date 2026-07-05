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

#[derive(Clone, Default)]
pub(crate) struct ScreenSessionBus {
    inner: Arc<Mutex<ScreenSessionState>>,
}

#[derive(Default)]
struct ScreenSessionState {
    replay: Vec<u8>,
    subscribers: Vec<mpsc::Sender<ScreenSessionEvent>>,
}

impl ScreenSessionBus {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn subscribe(&self) -> mpsc::Receiver<ScreenSessionEvent> {
        self.subscribe_with_replay().1
    }

    pub(crate) fn subscribe_with_replay(&self) -> (Vec<u8>, mpsc::Receiver<ScreenSessionEvent>) {
        let (tx, rx) = mpsc::channel();
        let replay = if let Ok(mut state) = self.inner.lock() {
            let replay = state.replay.clone();
            state.subscribers.push(tx);
            replay
        } else {
            Vec::new()
        };

        (replay, rx)
    }

    pub(crate) fn replay_snapshot(&self) -> Vec<u8> {
        self.inner
            .lock()
            .map(|state| state.replay.clone())
            .unwrap_or_default()
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
        self.publish(ScreenSessionEvent::Resize { cols, rows }, |_| {});
    }

    pub(crate) fn publish_detach(&self) {
        self.publish(ScreenSessionEvent::Detach, |_| {});
    }

    pub(crate) fn publish_exit(&self, code: i32) {
        self.publish(ScreenSessionEvent::Exit(code), |_| {});
    }

    fn publish(
        &self,
        event: ScreenSessionEvent,
        update: impl FnOnce(&mut ScreenSessionState),
    ) {
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
        let (replay, rx) = bus.subscribe_with_replay();
        bus.publish_output(b"!");

        assert_eq!(replay, b"hello".to_vec());
        assert_eq!(rx.try_recv(), Ok(ScreenSessionEvent::Output(b"!".to_vec())));
    }

    #[test]
    fn publishes_output_to_subscribers() {
        let bus = ScreenSessionBus::new();
        let rx = bus.subscribe();
        bus.publish_output(b"x");

        assert_eq!(rx.try_recv(), Ok(ScreenSessionEvent::Output(b"x".to_vec())));
    }

    #[test]
    fn models_attach_control_events() {
        assert_eq!(
            ScreenControlEvent::Input(b"x".to_vec()),
            ScreenControlEvent::Input(b"x".to_vec())
        );
    }
}