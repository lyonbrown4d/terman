pub(super) const DEFAULT_SCROLLBACK_LINES: usize = 1_000;

const DEFAULT_SCROLLBACK_COLS: usize = 80;
const MIN_SCROLLBACK_BYTES: usize = 4 * 1024;

#[derive(Clone)]
pub(super) struct ScreenReplayBuffer {
    bytes: Vec<u8>,
    scrollback_lines: usize,
}

impl Default for ScreenReplayBuffer {
    fn default() -> Self {
        Self {
            bytes: Vec::new(),
            scrollback_lines: DEFAULT_SCROLLBACK_LINES,
        }
    }
}

impl ScreenReplayBuffer {
    pub(super) fn snapshot(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    pub(super) fn len(&self) -> usize {
        self.bytes.len()
    }

    pub(super) fn scrollback_lines(&self) -> usize {
        self.scrollback_lines
    }

    pub(super) fn clear(&mut self) {
        self.bytes.clear();
    }

    pub(super) fn set_scrollback_lines(&mut self, lines: usize, cols: Option<u16>) {
        self.scrollback_lines = lines;
        self.trim(cols);
    }

    pub(super) fn append(&mut self, bytes: &[u8], cols: Option<u16>) {
        self.bytes.extend_from_slice(bytes);
        self.trim(cols);
    }

    pub(super) fn trim_to_cols(&mut self, cols: Option<u16>) {
        self.trim(cols);
    }

    fn trim(&mut self, cols: Option<u16>) {
        let max_bytes = max_replay_bytes(self.scrollback_lines, cols);
        if self.bytes.len() > max_bytes {
            let overflow = self.bytes.len() - max_bytes;
            self.bytes.drain(..overflow);
        }
    }
}

fn max_replay_bytes(scrollback_lines: usize, cols: Option<u16>) -> usize {
    if scrollback_lines == 0 {
        return 0;
    }
    let cols = cols.map(usize::from).unwrap_or(DEFAULT_SCROLLBACK_COLS);
    scrollback_lines
        .saturating_mul(cols)
        .max(MIN_SCROLLBACK_BYTES)
}

#[cfg(test)]
mod tests {
    use super::ScreenReplayBuffer;

    #[test]
    fn keeps_recent_bytes_with_default_minimum() {
        let mut replay = ScreenReplayBuffer::default();
        replay.set_scrollback_lines(1, Some(2));
        replay.append(&vec![b'x'; 5_000], Some(2));

        assert_eq!(replay.len(), 4 * 1024);
    }

    #[test]
    fn supports_zero_scrollback() {
        let mut replay = ScreenReplayBuffer::default();
        replay.set_scrollback_lines(0, Some(80));
        replay.append(b"hello", Some(80));

        assert!(replay.snapshot().is_empty());
    }
}