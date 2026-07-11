use std::ops::Range;

#[derive(Default)]
pub(crate) struct MouseWindowListState {
    window_list_start: Option<u16>,
    window_indexes: Vec<usize>,
    visible_entries: Vec<(usize, u16)>,
    cursor: usize,
    suppress_button_release: bool,
}

impl MouseWindowListState {
    pub(crate) fn sync_windows(&mut self, indexes: Vec<usize>, selected: usize) {
        let preferred = self.selected_window().unwrap_or(selected);
        self.window_indexes = indexes;
        self.cursor = self.window_indexes.iter()
            .position(|index| *index == preferred)
            .or_else(|| self.window_indexes.iter().position(|index| *index == selected))
            .unwrap_or(0);
    }

    pub(crate) fn set_visible_entries(&mut self, start: u16, entries: Vec<(usize, u16)>) {
        self.window_list_start = Some(start);
        self.visible_entries = entries;
    }

    pub(crate) fn clear(&mut self) {
        self.window_list_start = None;
        self.window_indexes.clear();
        self.visible_entries.clear();
        self.cursor = 0;
    }

    pub(crate) fn list_open(&self) -> bool {
        self.window_list_start.is_some()
    }

    pub(crate) fn selected_window(&self) -> Option<usize> {
        self.window_indexes.get(self.cursor).copied()
    }

    pub(crate) fn select_window(&mut self, index: usize) -> bool {
        let Some(position) = self.window_indexes.iter().position(|candidate| *candidate == index) else {
            return false;
        };
        self.cursor = position;
        true
    }

    pub(crate) fn move_selection(&mut self, amount: isize) {
        if self.window_indexes.is_empty() {
            return;
        }
        self.cursor = if amount < 0 {
            self.cursor.saturating_sub((-amount) as usize)
        } else {
            self.cursor.saturating_add(amount as usize)
                .min(self.window_indexes.len() - 1)
        };
    }

    pub(crate) fn select_first(&mut self) {
        self.cursor = 0;
    }

    pub(crate) fn select_last(&mut self) {
        self.cursor = self.window_indexes.len().saturating_sub(1);
    }

    pub(crate) fn visible_range(&self, capacity: usize) -> Range<usize> {
        let capacity = capacity.max(1).min(self.window_indexes.len());
        if capacity == 0 {
            return 0..0;
        }
        let start = self.cursor.saturating_sub(capacity / 2)
            .min(self.window_indexes.len() - capacity);
        start..start + capacity
    }

    pub(crate) fn suppress_button_release(&mut self) {
        self.suppress_button_release = true;
    }

    pub(crate) fn take_suppressed_button_release(&mut self) -> bool {
        let suppress = self.suppress_button_release;
        self.suppress_button_release = false;
        suppress
    }

    pub(crate) fn window_at(&self, row: u16, column: u16) -> Option<usize> {
        let start = self.window_list_start?;
        let offset = row.checked_sub(start)? as usize;
        let (index, width) = self.visible_entries.get(offset).copied()?;
        (column < width).then_some(index)
    }
}