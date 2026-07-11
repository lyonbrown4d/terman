use super::ScreenSessionBus;

impl ScreenSessionBus {
    pub(crate) fn set_wrap_enabled(&self, requested: Option<bool>) {
        let setting = self.inner.lock().ok().and_then(|mut state| {
            let index = state.active_window;
            let cols = state.cols;
            let window = state.window_mut(index)?;
            let enabled = requested.unwrap_or_else(|| !window.wrap_enabled());
            window.set_wrap_enabled(enabled, cols);
            Some((index, enabled, window.wrap_control().to_vec()))
        });
        let Some((index, enabled, control)) = setting else {
            return;
        };
        self.publish_transient_output(&control);
        let message = terman_common::builtin_screen_wrap_status_hint(index, enabled);
        self.publish_message(format!("\r\n{message}\r\n").as_bytes());
    }
}
