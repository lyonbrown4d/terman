use super::ScreenSessionBus;

const DEFAULT_REGISTER: &str = ".";

impl ScreenSessionBus {
    pub(crate) fn set_register(&self, name: String, bytes: Vec<u8>) {
        if let Ok(mut state) = self.inner.lock() {
            if name == DEFAULT_REGISTER {
                state.paste_buffer = bytes.clone();
            }
            state.registers.insert(name, bytes);
        }
    }

    pub(crate) fn register_snapshot(&self, name: &str) -> Vec<u8> {
        self.inner
            .lock()
            .ok()
            .map(|state| {
                if name == DEFAULT_REGISTER {
                    state.paste_buffer.clone()
                } else {
                    state.registers.get(name).cloned().unwrap_or_default()
                }
            })
            .unwrap_or_default()
    }
}