use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub(crate) const SCREEN_CONTROL_PREFIX: u8 = 0x01;

pub(crate) fn key_to_bytes(key: KeyEvent) -> Option<Vec<u8>> {
    if key.kind != KeyEventKind::Press { return None; }

    if key.modifiers.contains(KeyModifiers::CONTROL) {
        if let KeyCode::Char(c) = key.code {
            return ctrl_char_bytes(c);
        }
    }

    match key.code {
        KeyCode::Char(c) => char_bytes(c, key.modifiers),
        KeyCode::Enter => Some(b"\r".to_vec()),
        KeyCode::Tab => Some(b"\t".to_vec()),
        KeyCode::Backspace => Some(vec![0x08]),
        KeyCode::Esc => Some(vec![0x1b]),
        KeyCode::Up => Some(ansi_bytes("\x1b[A")),
        KeyCode::Down => Some(ansi_bytes("\x1b[B")),
        KeyCode::Right => Some(ansi_bytes("\x1b[C")),
        KeyCode::Left => Some(ansi_bytes("\x1b[D")),
        KeyCode::Home => Some(ansi_bytes("\x1b[H")),
        KeyCode::End => Some(ansi_bytes("\x1b[F")),
        KeyCode::PageUp => Some(ansi_bytes("\x1b[5~")),
        KeyCode::PageDown => Some(ansi_bytes("\x1b[6~")),
        KeyCode::Insert => Some(ansi_bytes("\x1b[2~")),
        KeyCode::Delete => Some(ansi_bytes("\x1b[3~")),
        _ => None,
    }
}

fn char_bytes(c: char, modifiers: KeyModifiers) -> Option<Vec<u8>> {
    if modifiers.contains(KeyModifiers::ALT) {
        let mut bytes = vec![0x1b];
        bytes.extend_from_slice(c.to_string().as_bytes());
        Some(bytes)
    } else {
        Some(c.to_string().into_bytes())
    }
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

fn ansi_bytes(sequence: &str) -> Vec<u8> {
    sequence.as_bytes().to_vec()
}