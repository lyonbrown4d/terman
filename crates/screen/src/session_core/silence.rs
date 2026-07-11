use std::time::Instant;

use super::{ScreenSessionBus, ScreenSessionEvent};

pub(crate) const DEFAULT_SILENCE_SECONDS: u64 = 30;

impl ScreenSessionBus {
    pub(crate) fn set_silence_seconds(&self, seconds: Option<u64>) {
        let seconds = seconds.filter(|seconds| *seconds > 0);
        let setting = self.inner.lock().ok().and_then(|mut state| {
            let index = state.active_window;
            state.window_mut(index)?.set_silence_seconds(seconds);
            Some((index, seconds))
        });
        if let Some((index, seconds)) = setting {
            self.publish_silence_status(index, seconds);
        }
    }

    pub(crate) fn toggle_silence(&self) {
        let setting = self.inner.lock().ok().and_then(|mut state| {
            let index = state.active_window;
            let window = state.window_mut(index)?;
            let seconds = if window.silence_seconds().is_some() {
                None
            } else {
                Some(DEFAULT_SILENCE_SECONDS)
            };
            window.set_silence_seconds(seconds);
            Some((index, seconds))
        });
        if let Some((index, seconds)) = setting {
            self.publish_silence_status(index, seconds);
        }
    }

    pub(crate) fn poll_silence(&self) {
        let Ok(mut state) = self.inner.lock() else {
            return;
        };
        let now = Instant::now();
        let notifications = state
            .windows
            .iter_mut()
            .filter_map(|window| {
                window.take_silence_notification(now).map(|seconds| {
                    (window.index(), window.title().map(str::to_owned), seconds)
                })
            })
            .collect::<Vec<_>>();

        for (index, title, seconds) in notifications {
            let message = terman_common::builtin_screen_silence_activity_hint(
                index,
                title.as_deref(),
                seconds,
            );
            let record = format!("\r\n{message}\r\n").into_bytes();
            state.last_message = record.clone();
            let mut notification = Vec::with_capacity(record.len() + 1);
            notification.push(0x07);
            notification.extend(record);
            let event = ScreenSessionEvent::Output(notification);
            state
                .subscribers
                .retain(|subscriber| subscriber.sender.send(event.clone()).is_ok());
        }
    }

    fn publish_silence_status(&self, index: usize, seconds: Option<u64>) {
        let message = terman_common::builtin_screen_silence_status_hint(index, seconds);
        self.publish_message(format!("\r\n{message}\r\n").as_bytes());
    }
}