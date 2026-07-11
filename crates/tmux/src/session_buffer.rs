use crate::{
    ipc::TmuxBufferInfo,
    session_core::TmuxSessionBus,
    session_state::{TmuxPasteBuffer, TmuxSessionState},
};

const MAX_BUFFERS: usize = 50;

impl TmuxSessionBus {
    pub(crate) fn set_buffer(&self, name: Option<String>, bytes: Vec<u8>) -> bool {
        let Ok(mut state) = self.inner.lock() else {
            return false;
        };
        let name = normalize_name(name).unwrap_or_else(|| next_buffer_name(&mut state));
        state.buffers.retain(|buffer| buffer.name != name);
        state.buffers.insert(0, TmuxPasteBuffer { name, bytes });
        state.buffers.truncate(MAX_BUFFERS);
        true
    }

    pub(crate) fn buffer_snapshot(&self, name: Option<&str>) -> Option<TmuxBufferInfo> {
        let state = self.inner.lock().ok()?;
        select_buffer(&state, name).map(buffer_info)
    }

    pub(crate) fn buffers_snapshot(&self) -> Vec<TmuxBufferInfo> {
        self.inner
            .lock()
            .map(|state| state.buffers.iter().map(buffer_info).collect())
            .unwrap_or_default()
    }

    pub(crate) fn delete_buffer(&self, name: Option<&str>) -> bool {
        let Ok(mut state) = self.inner.lock() else {
            return false;
        };
        let Some(index) = buffer_index(&state, name) else {
            return false;
        };
        state.buffers.remove(index);
        true
    }
}

fn normalize_name(name: Option<String>) -> Option<String> {
    name.map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
}

fn next_buffer_name(state: &mut TmuxSessionState) -> String {
    loop {
        let name = format!("buffer{}", state.next_buffer);
        state.next_buffer = state.next_buffer.saturating_add(1);
        if !state.buffers.iter().any(|buffer| buffer.name == name) {
            return name;
        }
    }
}

fn select_buffer<'a>(
    state: &'a TmuxSessionState,
    name: Option<&str>,
) -> Option<&'a TmuxPasteBuffer> {
    buffer_index(state, name).and_then(|index| state.buffers.get(index))
}

fn buffer_index(state: &TmuxSessionState, name: Option<&str>) -> Option<usize> {
    match name.filter(|name| !name.trim().is_empty()) {
        Some(name) => state.buffers.iter().position(|buffer| buffer.name == name),
        None => (!state.buffers.is_empty()).then_some(0),
    }
}

fn buffer_info(buffer: &TmuxPasteBuffer) -> TmuxBufferInfo {
    TmuxBufferInfo {
        name: buffer.name.clone(),
        bytes: buffer.bytes.clone(),
    }
}