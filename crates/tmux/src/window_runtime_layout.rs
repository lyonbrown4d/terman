use super::TmuxWindowRuntime;

impl TmuxWindowRuntime {
    pub(crate) fn cycle_pane_layout(&mut self) -> bool {
        let changed = self
            .view
            .lock()
            .map(|mut view| view.cycle_layout().is_some())
            .unwrap_or(false);
        if changed {
            self.resize_from_view();
            self.publish_frame();
        }
        changed
    }
}