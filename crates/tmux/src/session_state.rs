use std::sync::mpsc;

use crate::session_model::TmuxSessionEvent;

const MAX_REPLAY_BYTES: usize = 64 * 1024;

pub(crate) struct TmuxSessionSubscriber {
    pub(crate) client_id: Option<String>,
    pub(crate) sender: mpsc::Sender<TmuxSessionEvent>,
}

pub(crate) struct TmuxPaneReplay {
    pub(crate) index: u32,
    pub(crate) capture: Vec<u8>,
}

pub(crate) struct TmuxWindowReplay {
    pub(crate) index: u32,
    pub(crate) name: String,
    pub(crate) replay: Vec<u8>,
    pub(crate) active_pane: u32,
    pub(crate) panes: Vec<TmuxPaneReplay>,
}

pub(crate) struct TmuxSessionState {
    pub(crate) windows: Vec<TmuxWindowReplay>,
    pub(crate) subscribers: Vec<TmuxSessionSubscriber>,
    pub(crate) attached_clients: u32,
    pub(crate) active_window: u32,
    pub(crate) last_window: Option<u32>,
    pub(crate) cols: Option<u16>,
    pub(crate) rows: Option<u16>,
    pub(crate) buffer: Vec<u8>,
}

impl TmuxSessionState {
    pub(crate) fn new(windows: u32) -> Self {
        let mut state = Self {
            windows: Vec::new(),
            subscribers: Vec::new(),
            attached_clients: 0,
            active_window: 0,
            last_window: None,
            cols: None,
            rows: None,
            buffer: Vec::new(),
        };
        state.set_window_count(windows.max(1));
        state
    }

    pub(crate) fn active_replay(&self) -> &[u8] {
        self.windows
            .iter()
            .find(|window| window.index == self.active_window)
            .map(|window| window.replay.as_slice())
            .unwrap_or(&[])
    }

    pub(crate) fn active_replay_mut(&mut self) -> &mut Vec<u8> {
        self.ensure_window(self.active_window, self.active_window.to_string());
        self.windows
            .iter_mut()
            .find(|window| window.index == self.active_window)
            .map(|window| &mut window.replay)
            .expect("active window exists")
    }

    pub(crate) fn window_capture(&self, window: u32, pane: Option<u32>) -> Option<&[u8]> {
        let window = self.windows.iter().find(|item| item.index == window)?;
        let pane = pane.unwrap_or(window.active_pane);
        window
            .panes
            .iter()
            .find(|item| item.index == pane)
            .map(|item| item.capture.as_slice())
    }

    pub(crate) fn clear_window_capture(&mut self, window: u32, pane: Option<u32>) -> bool {
        let Some(window) = self.windows.iter_mut().find(|item| item.index == window) else {
            return false;
        };
        let pane = pane.unwrap_or(window.active_pane);
        let Some(pane) = window.panes.iter_mut().find(|item| item.index == pane) else {
            return false;
        };
        pane.capture.clear();
        true
    }

    pub(crate) fn pane_status(&self, window: u32) -> Option<(String, u32, Vec<u32>)> {
        let window = self.windows.iter().find(|item| item.index == window)?;
        Some((
            window.name.clone(),
            window.active_pane,
            window.panes.iter().map(|pane| pane.index).collect(),
        ))
    }

    pub(crate) fn replace_window_output(
        &mut self,
        index: u32,
        replay: Vec<u8>,
        active_pane: u32,
        captures: Vec<(u32, Vec<u8>)>,
    ) {
        self.ensure_window(index, index.to_string());
        let Some(window) = self.windows.iter_mut().find(|window| window.index == index) else {
            return;
        };
        window.replay = replay;
        window.active_pane = active_pane;
        window.panes = captures
            .into_iter()
            .map(|(index, capture)| TmuxPaneReplay { index, capture })
            .collect();
        if !window.panes.iter().any(|pane| pane.index == active_pane) {
            window.panes.push(TmuxPaneReplay {
                index: active_pane,
                capture: Vec::new(),
            });
        }
        window.panes.sort_by_key(|pane| pane.index);
    }

    pub(crate) fn has_window(&self, index: u32) -> bool {
        self.windows.iter().any(|window| window.index == index)
    }

    pub(crate) fn ensure_window(&mut self, index: u32, name: String) {
        if !self.has_window(index) {
            self.windows.push(TmuxWindowReplay {
                index,
                name,
                replay: Vec::new(),
                active_pane: 0,
                panes: vec![TmuxPaneReplay {
                    index: 0,
                    capture: Vec::new(),
                }],
            });
            self.windows.sort_by_key(|window| window.index);
        }
    }

    pub(crate) fn append_window_output(&mut self, index: u32, bytes: &[u8]) {
        self.ensure_window(index, index.to_string());
        let window = self
            .windows
            .iter_mut()
            .find(|window| window.index == index)
            .expect("window exists");
        window.replay.extend_from_slice(bytes);
        trim_replay(&mut window.replay);
        let active = window.active_pane;
        if let Some(pane) = window.panes.iter_mut().find(|pane| pane.index == active) {
            pane.capture.extend_from_slice(bytes);
            trim_replay(&mut pane.capture);
        }
    }

    pub(crate) fn set_window_count(&mut self, windows: u32) {
        for index in 0..windows {
            self.ensure_window(index, index.to_string());
        }
        self.windows.retain(|window| window.index < windows);
        if !self.has_window(self.active_window) {
            self.active_window = self.windows.first().map(|window| window.index).unwrap_or(0);
        }
    }

    pub(crate) fn remove_window(&mut self, index: u32) {
        self.windows.retain(|window| window.index != index);
        if self.last_window == Some(index) {
            self.last_window = None;
        }
        if self.windows.is_empty() {
            self.ensure_window(0, String::from("0"));
        }
        if !self.has_window(self.active_window) {
            self.active_window = self.windows.first().map(|window| window.index).unwrap_or(0);
        }
    }
}

fn trim_replay(bytes: &mut Vec<u8>) {
    if bytes.len() > MAX_REPLAY_BYTES {
        let overflow = bytes.len() - MAX_REPLAY_BYTES;
        bytes.drain(..overflow);
    }
}

pub(crate) fn select_window_state(state: &mut TmuxSessionState, index: u32) {
    if state.active_window != index {
        state.last_window = Some(state.active_window);
    }
    state.active_window = index;
}

pub(crate) fn send_to_subscribers(state: &mut TmuxSessionState, event: TmuxSessionEvent) {
    state
        .subscribers
        .retain(|subscriber| subscriber.sender.send(event.clone()).is_ok());
}
