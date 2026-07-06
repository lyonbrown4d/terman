use super::{ScreenSessionStatus, ScreenWindowStatus, replay::DEFAULT_SCROLLBACK_LINES, window::ScreenWindowState};

pub(crate) struct ScreenRemovedWindow {
    pub(crate) active_window: Option<usize>,
    pub(crate) replay: Vec<u8>,
    pub(crate) last_window: bool,
    pub(crate) redraw: bool,
}

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
        self.window(self.active_window)
    }

    pub(super) fn active_window_mut(&mut self) -> Option<&mut ScreenWindowState> {
        self.window_mut(self.active_window)
    }

    pub(super) fn window_mut(&mut self, index: usize) -> Option<&mut ScreenWindowState> {
        self.windows.iter_mut().find(|window| window.index() == index)
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
        let replay = self.window(index)?.replay_snapshot();
        self.active_window = index;
        Some(replay)
    }

    pub(super) fn remove_window(&mut self, index: usize) -> Option<ScreenRemovedWindow> {
        let position = self.windows.iter().position(|window| window.index() == index)?;
        let was_active = self.active_window == index;
        self.windows.remove(position);
        if self.windows.is_empty() {
            return Some(ScreenRemovedWindow {
                active_window: None,
                replay: Vec::new(),
                last_window: true,
                redraw: false,
            });
        }

        let active_missing = self.window(self.active_window).is_none();
        if was_active || active_missing {
            let next_position = position.min(self.windows.len() - 1);
            self.active_window = self.windows[next_position].index();
        }
        let replay = self
            .active_window()
            .map(ScreenWindowState::replay_snapshot)
            .unwrap_or_default();
        Some(ScreenRemovedWindow {
            active_window: Some(self.active_window),
            replay,
            last_window: false,
            redraw: was_active,
        })
    }

    fn window(&self, index: usize) -> Option<&ScreenWindowState> {
        self.windows.iter().find(|window| window.index() == index)
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