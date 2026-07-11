use crate::pane_layout::PaneDirection;

use super::TmuxWindowRuntime;

impl TmuxWindowRuntime {
    pub(crate) fn resize_pane_direction(
        &mut self,
        index: u32,
        direction: PaneDirection,
        adjustment: u16,
    ) -> bool {
        let changed = self.view.lock()
            .map(|mut view| view.resize_pane_direction(index, direction, adjustment))
            .unwrap_or(false);
        if changed {
            self.resize_from_view();
            self.publish_frame();
        }
        changed
    }
}