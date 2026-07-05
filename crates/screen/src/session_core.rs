use std::{
    sync::{Arc, Mutex, mpsc},
};

const MAX_REPLAY_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ScreenSessionEvent {
    Output(Vec<u8>),
    Resize { cols: u16, rows: u16 },
    Exit(i32),
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
        let (tx, rx) = mpsc::channel();
        if let Ok(mut state) = self.inner.lock() {
            state.subscribers.push(tx);
        }
        rx
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
    use super::{ScreenSessionBus, ScreenSessionEvent};

    #[test]
    fn replays_recent_output_to_attach_clients() {
        let bus = ScreenSessionBus::new();
        bus.publish_output(b"hello");

        assert_eq!(bus.replay_snapshot(), b"hello".to_vec());
    }

    #[test]
    fn publishes_output_to_subscribers() {
        let bus = ScreenSessionBus::new();
        let rx = bus.subscribe();
        bus.publish_output(b"x");

        assert_eq!(rx.try_recv(), Ok(ScreenSessionEvent::Output(b"x".to_vec())));
    }
}