use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub(crate) fn key_event_bytes(key: KeyEvent) -> Option<Vec<u8>> {
    if key.kind != KeyEventKind::Press {
        return None;
    }

    match key.code {
        KeyCode::Backspace => Some(vec![0x7f]),
        KeyCode::Enter => Some(vec![b'\r']),
        KeyCode::Left => Some(ansi_bytes("\x1b[D")),
        KeyCode::Right => Some(ansi_bytes("\x1b[C")),
        KeyCode::Up => Some(ansi_bytes("\x1b[A")),
        KeyCode::Down => Some(ansi_bytes("\x1b[B")),
        KeyCode::Home => Some(ansi_bytes("\x1b[H")),
        KeyCode::End => Some(ansi_bytes("\x1b[F")),
        KeyCode::PageUp => Some(ansi_bytes("\x1b[5~")),
        KeyCode::PageDown => Some(ansi_bytes("\x1b[6~")),
        KeyCode::Tab => Some(vec![b'\t']),
        KeyCode::BackTab => Some(ansi_bytes("\x1b[Z")),
        KeyCode::Delete => Some(ansi_bytes("\x1b[3~")),
        KeyCode::Insert => Some(ansi_bytes("\x1b[2~")),
        KeyCode::Esc => Some(vec![0x1b]),
        KeyCode::Char(ch) => char_key_bytes(ch, key.modifiers),
        KeyCode::F(number) => function_key_bytes(number),
        _ => None,
    }
}

fn char_key_bytes(ch: char, modifiers: KeyModifiers) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    if modifiers.contains(KeyModifiers::ALT) {
        bytes.push(0x1b);
    }

    if modifiers.contains(KeyModifiers::CONTROL) {
        bytes.extend(control_char_bytes(ch)?);
    } else {
        bytes.extend(encoded_char_bytes(ch));
    }

    Some(bytes)
}

fn control_char_bytes(ch: char) -> Option<Vec<u8>> {
    let upper = ch.to_ascii_uppercase();
    if upper.is_ascii_uppercase() {
        return Some(vec![(upper as u8) - b'A' + 1]);
    }

    match ch {
        ' ' => Some(vec![0x00]),
        '[' => Some(vec![0x1b]),
        '\\' => Some(vec![0x1c]),
        ']' => Some(vec![0x1d]),
        '^' => Some(vec![0x1e]),
        '_' => Some(vec![0x1f]),
        '?' => Some(vec![0x7f]),
        _ => None,
    }
}

fn function_key_bytes(number: u8) -> Option<Vec<u8>> {
    match number {
        1 => Some(ansi_bytes("\x1bOP")),
        2 => Some(ansi_bytes("\x1bOQ")),
        3 => Some(ansi_bytes("\x1bOR")),
        4 => Some(ansi_bytes("\x1bOS")),
        5 => Some(ansi_bytes("\x1b[15~")),
        6 => Some(ansi_bytes("\x1b[17~")),
        7 => Some(ansi_bytes("\x1b[18~")),
        8 => Some(ansi_bytes("\x1b[19~")),
        9 => Some(ansi_bytes("\x1b[20~")),
        10 => Some(ansi_bytes("\x1b[21~")),
        11 => Some(ansi_bytes("\x1b[23~")),
        12 => Some(ansi_bytes("\x1b[24~")),
        _ => None,
    }
}

fn encoded_char_bytes(ch: char) -> Vec<u8> {
    let mut buffer = [0; 4];
    ch.encode_utf8(&mut buffer).as_bytes().to_vec()
}

fn ansi_bytes(sequence: &str) -> Vec<u8> {
    sequence.as_bytes().to_vec()
}