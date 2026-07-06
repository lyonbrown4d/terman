use super::{ScreenSessionStatus, ScreenWindowStatus, replay::DEFAULT_SCROLLBACK_LINES, window::ScreenWindowState};

pub(super) struct ScreenSessionSubscriber {
    pub(super) client_id: Option<String>,
    pub(super) sender: std::sync::mpsc::Sender<super::ScreenSessionEvent>,
}

pub(super) struct ScreenSessionState {
    pub(super) windows: Vec<ScreenWindowState>,
    pub(super) active_window: usize,
    pub(super) paste_buffer: Vec<u8>,
    pub(super) subscribers: Vec<ScreenSessionSubscriber>,
    pub(super) attach_clients: usize,
    pub(super) cols: Option<u16>,
    pub(super) rows: Option<u16>,
}

impl Default for ScreenSessionState {
    fn default() -> Self {
        Self {
            windows: vec![ScreenWindowState::new(0)],
            active_window: 0,
            paste_buffer: Vec::new(),
            subscribers: Vec::new(),
            attach_clients: 0,
            cols: None,
            rows: None,
        }
    }
}

impl ScreenSessionState {
    pub(super) fn active_window(&self) -> Option<&ScreenWindowState> {
        self.windows.get(self.active_window)
    }

    pub(super) fn active_window_mut(&mut self) -> Option<&mut ScreenWindowState> {
        self.windows.get_mut(self.active_window)
    }

    pub(super) fn add_window(&mut self, index: usize, title: Option<String>) {
        let mut window = ScreenWindowState::new(index);
        if let Some(title) = title {
            window.set_title(title);
        }
        self.windows.push(window);
        self.active_window = index;
    }

    pub(super) fn select_window(&mut self, index: usize) -> Option<Vec<u8>> {
        let window = self.windows.get(index)?;
        self.active_window = index;
        Some(window.replay_snapshot())
    }
}

pub(super) fn session_status(state: &ScreenSessionState) -> ScreenSessionStatus {
    let active = state.active_window();
    let replay_bytes = active.map(ScreenWindowState::replay_len).unwrap_or_default();
    let window_title = active.and_then(ScreenWindowState::title).map(str::to_string);
    let scrollback_lines = active
        .map(ScreenWindowState::scrollback_lines)
        .unwrap_or(DEFAULT_SCROLLBACK_LINES);
    ScreenSessionStatus {
        replay_bytes,
        attach_clients: state.attach_clients,
        cols: state.cols,
        rows: state.rows,
        scrollback_lines,
        window_title,
        active_window: state.active_window,
        windows: state
            .windows
            .iter()
            .map(|window| window.status(window.index() == state.active_window))
            .collect(),
    }
}

pub(super) fn fallback_status() -> ScreenSessionStatus {
    ScreenSessionStatus {
        replay_bytes: 0,
        attach_clients: 0,
        cols: None,
        rows: None,
        scrollback_lines: DEFAULT_SCROLLBACK_LINES,
        window_title: None,
        active_window: 0,
        windows: vec![ScreenWindowStatus {
            index: 0,
            title: None,
            active: true,
            replay_bytes: 0,
        }],
    }
}