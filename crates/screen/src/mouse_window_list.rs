#[derive(Default)]
pub(crate) struct MouseWindowListState {
    window_list_start: Option<u16>,
    window_entries: Vec<(usize, u16)>,
    suppress_button_release: bool,
}

impl MouseWindowListState {
    pub(crate) fn show_window_list(&mut self, start: u16, entries: Vec<(usize, u16)>) {
        self.window_list_start = Some(start);
        self.window_entries = entries;
    }

    pub(crate) fn clear(&mut self) {
        self.window_list_start = None;
        self.window_entries.clear();
    }

    pub(crate) fn list_open(&self) -> bool {
        self.window_list_start.is_some()
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
        let (index, width) = self.window_entries.get(offset).copied()?;
        (column < width).then_some(index)
    }
}