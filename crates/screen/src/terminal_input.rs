use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

const SCREEN_CONTROL_PREFIX: u8 = 0x01;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ScreenInputAction {
    Bytes(Vec<u8>),
    Detach,
}

#[derive(Default)]
pub(crate) struct ScreenInputDecoder {
    pending_prefix: bool,
}

impl ScreenInputDecoder {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn decode_key(&mut self, key: KeyEvent) -> Option<ScreenInputAction> {
        if key.kind != KeyEventKind::Press {
            return None;
        }

        if self.pending_prefix {
            self.pending_prefix = false;
            return self.decode_prefixed_key(key);
        }

        if is_screen_prefix_key(key) {
            self.pending_prefix = true;
            return None;
        }

        key_to_bytes(key).map(ScreenInputAction::Bytes)
    }

    fn decode_prefixed_key(&mut self, key: KeyEvent) -> Option<ScreenInputAction> {
        match key.code {
            KeyCode::Char('d') | KeyCode::Char('D') if key.modifiers.is_empty() => {
                Some(ScreenInputAction::Detach)
            }
            _ if is_screen_prefix_key(key) => {
                Some(ScreenInputAction::Bytes(vec![SCREEN_CONTROL_PREFIX]))
            }
            _ => {
                let mut bytes = vec![SCREEN_CONTROL_PREFIX];
                if let Some(mut key_bytes) = key_to_bytes(key) {
                    bytes.append(&mut key_bytes);
                }
                Some(ScreenInputAction::Bytes(bytes))
            }
        }
    }
}

pub(crate) fn key_to_bytes(key: KeyEvent) -> Option<Vec<u8>> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        if let KeyCode::Char(c) = key.code {
            return ctrl_char_bytes(c);
        }
    }

    match key.code {
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::ALT) {
                let mut bytes = vec![0x1b];
                bytes.extend_from_slice(c.to_string().as_bytes());
                Some(bytes)
            } else {
                Some(c.to_string().into_bytes())
            }
        }
        KeyCode::Enter => Some(b"\r".to_vec()),
        KeyCode::Tab => Some(b"\t".to_vec()),
        KeyCode::Backspace => Some(vec![0x08]),
        KeyCode::Esc => Some(vec![0x1b]),
        KeyCode::Up => Some(vec![0x1b, b'[', b'A']),
        KeyCode::Down => Some(vec![0x1b, b'[', b'B']),
        KeyCode::Right => Some(vec![0x1b, b'[', b'C']),
        KeyCode::Left => Some(vec![0x1b, b'[', b'D']),
        KeyCode::Home => Some(vec![0x1b, b'[', b'H']),
        KeyCode::End => Some(vec![0x1b, b'[', b'F']),
        KeyCode::PageUp => Some(vec![0x1b, b'[', b'5', b'~']),
        KeyCode::PageDown => Some(vec![0x1b, b'[', b'6', b'~']),
        KeyCode::Insert => Some(vec![0x1b, b'[', b'2', b'~']),
        KeyCode::Delete => Some(vec![0x1b, b'[', b'3', b'~']),
        _ => None,
    }
}

fn is_screen_prefix_key(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('a') | KeyCode::Char('A'))
        && key.modifiers.contains(KeyModifiers::CONTROL)
}

fn ctrl_char_bytes(c: char) -> Option<Vec<u8>> {
    let lower = c.to_ascii_lowercase();
    let b = match lower {
        'a' => SCREEN_CONTROL_PREFIX,
        'b' => 0x02,
        'c' => 0x03,
        'd' => 0x04,
        'e' => 0x05,
        'f' => 0x06,
        'g' => 0x07,
        'h' => 0x08,
        'i' => 0x09,
        'j' => 0x0a,
        'k' => 0x0b,
        'l' => 0x0c,
        'm' => 0x0d,
        'n' => 0x0e,
        'o' => 0x0f,
        'p' => 0x10,
        'q' => 0x11,
        'r' => 0x12,
        's' => 0x13,
        't' => 0x14,
        'u' => 0x15,
        'v' => 0x16,
        'w' => 0x17,
        'x' => 0x18,
        'y' => 0x19,
        'z' => 0x1a,
        '[' => 0x1b,
        '\\' => 0x1c,
        ']' => 0x1d,
        '^' => 0x1e,
        '_' => 0x1f,
        _ => return None,
    };
    Some(vec![b])
}

#[cfg(test)]
mod tests {
    use super::{ScreenInputAction, ScreenInputDecoder, key_to_bytes};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn maps_control_char_to_control_byte() {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);

        assert_eq!(key_to_bytes(key), Some(vec![0x03]));
    }

    #[test]
    fn maps_arrow_key_to_escape_sequence() {
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());

        assert_eq!(key_to_bytes(key), Some(vec![0x1b, b'[', b'A']));
    }

    #[test]
    fn detects_screen_detach_prefix() {
        let mut decoder = ScreenInputDecoder::new();
        let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        let detach = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty());

        assert_eq!(decoder.decode_key(prefix), None);
        assert_eq!(decoder.decode_key(detach), Some(ScreenInputAction::Detach));
    }

    #[test]
    fn detects_screen_kill_prefix() {
        let mut decoder = ScreenInputDecoder::new();
        let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        let kill = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty());

        assert_eq!(decoder.decode_key(prefix), None);
        assert_eq!(decoder.decode_key(kill), Some(ScreenInputAction::Kill));
    }

    #[test]
    fn sends_literal_prefix_when_prefix_is_repeated() {
        let mut decoder = ScreenInputDecoder::new();
        let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);

        assert_eq!(decoder.decode_key(prefix), None);
        assert_eq!(
            decoder.decode_key(prefix),
            Some(ScreenInputAction::Bytes(vec![0x01]))
        );
    }
}