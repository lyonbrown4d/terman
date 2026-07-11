use super::{ScreenSessionBus, ScreenSessionEvent};

impl ScreenSessionBus {
    pub(crate) fn set_monitor_enabled(&self, requested: Option<bool>) {
        let setting = self.inner.lock().ok().and_then(|mut state| {
            let index = state.active_window;
            let window = state.window_mut(index)?;
            let enabled = requested.unwrap_or_else(|| !window.monitor_enabled());
            window.set_monitor_enabled(enabled);
            Some((index, enabled))
        });
        let Some((index, enabled)) = setting else {
            return;
        };
        let message = terman_common::builtin_screen_monitor_status_hint(index, enabled);
        self.publish_message(format!("\r\n{message}\r\n").as_bytes());
    }

    pub(crate) fn publish_window_output(&self, index: usize, bytes: &[u8]) {
        let Ok(mut state) = self.inner.lock() else {
            return;
        };
        let cols = state.cols;
        let active = state.active_window == index;
        let activity_title = {
            let Some(window) = state.window_mut(index) else {
                return;
            };
            window.append_output(bytes, cols);
            if !active && window.mark_activity() {
                Some(window.title().map(str::to_owned))
            } else {
                None
            }
        };

        if active {
            let event = ScreenSessionEvent::Output(bytes.to_vec());
            state
                .subscribers
                .retain(|subscriber| subscriber.sender.send(event.clone()).is_ok());
            return;
        }
        let Some(title) = activity_title else {
            return;
        };

        let message =
            terman_common::builtin_screen_monitor_activity_hint(index, title.as_deref());
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