use crate::region_types::{BLANK_SCREEN_WINDOW_INDEX, ScreenRegionAxis, ScreenRegionFocus};

use super::ScreenSessionBus;

impl ScreenSessionBus {
    pub(crate) fn blank_region(&self) -> Option<(usize, Vec<u8>)> {
        let frame = {
            let mut state = self.inner.lock().ok()?;
            state.blank_region();
            state.render_regions()
        };
        self.publish_transient_output(&frame);
        Some((BLANK_SCREEN_WINDOW_INDEX, frame))
    }
    pub(crate) fn split_region(
        &self,
        axis: ScreenRegionAxis,
    ) -> Option<(usize, Vec<u8>)> {
        let (active, frame) = {
            let mut state = self.inner.lock().ok()?;
            if !state.split_region(axis) {
                return None;
            }
            (state.active_window, state.render_regions())
        };
        self.publish_transient_output(&frame);
        Some((active, frame))
    }

    pub(crate) fn focus_region(
        &self,
        target: ScreenRegionFocus,
    ) -> Option<(usize, Vec<u8>)> {
        let (active, frame) = {
            let mut state = self.inner.lock().ok()?;
            if !state.has_multiple_regions() {
                return None;
            }
            let active = state.focus_region(target)?;
            (active, state.render_regions())
        };
        self.publish_transient_output(&frame);
        Some((active, frame))
    }

    pub(crate) fn remove_region(&self) -> Option<(usize, Vec<u8>)> {
        let (active, frame) = {
            let mut state = self.inner.lock().ok()?;
            let active = state.remove_region()?;
            (active, state.render_regions())
        };
        self.publish_transient_output(&frame);
        Some((active, frame))
    }

    pub(crate) fn only_region(&self) -> Option<(usize, Vec<u8>)> {
        let (active, frame) = {
            let mut state = self.inner.lock().ok()?;
            let active = state.only_region()?;
            (active, state.render_regions())
        };
        self.publish_transient_output(&frame);
        Some((active, frame))
    }

    pub(crate) fn publish_region_redraw(&self) -> Option<Vec<u8>> {
        let frame = {
            let state = self.inner.lock().ok()?;
            (state.has_multiple_regions() || state.active_window == BLANK_SCREEN_WINDOW_INDEX).then(|| state.render_regions())?
        };
        self.publish_transient_output(&frame);
        Some(frame)
    }

    pub(crate) fn publish_region_redraw_for_output(
        &self,
        window_index: usize,
    ) -> Option<Vec<u8>> {
        let frame = {
            let state = self.inner.lock().ok()?;
            if !state.has_multiple_regions() || !state.region_contains_window(window_index) {
                return None;
            }
            state.render_regions()
        };
        self.publish_transient_output(&frame);
        Some(frame)
    }
}