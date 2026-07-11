use crate::pane_layout::PaneDirection;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub(crate) fn is_key_press(key: &KeyEvent) -> bool {
    key.kind == KeyEventKind::Press
}

pub(crate) fn is_tmux_prefix_key(key: &KeyEvent) -> bool {
    is_key_press(key)
        && matches!(&key.code, KeyCode::Char('b') | KeyCode::Char('B'))
        && key.modifiers.contains(KeyModifiers::CONTROL)
        && !key.modifiers.contains(KeyModifiers::ALT)
}

pub(crate) fn is_detach_key(key: &KeyEvent) -> bool {
    is_key_press(key)
        && matches!(&key.code, KeyCode::Char('d'))
        && !key.modifiers.contains(KeyModifiers::CONTROL)
        && !key.modifiers.contains(KeyModifiers::ALT)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TmuxPrefixCommand {
    NextWindow,
    PreviousWindow,
    CreateWindow,
    KillWindow,
    FindWindow,
    RenameSession,
    RenameWindow,
    ListWindows,
    LastWindow,
    SplitHorizontal,
    SplitVertical,
    NextPane,
    LastPane,
    DisplayPanes,
    SelectPane(PaneDirection),
    ResizePane(PaneDirection),
    SwapPaneUp,
    SwapPaneDown,
    TogglePaneZoom,
    CopyMode,
    PasteBuffer,
    KillPane,
    CommandPrompt,
    Help,
    SelectWindow(u32),
}

pub(crate) fn tmux_prefix_command(key: &KeyEvent) -> Option<TmuxPrefixCommand> {
    if !is_key_press(key) || key.modifiers.contains(KeyModifiers::ALT) {
        return None;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Left => Some(TmuxPrefixCommand::ResizePane(PaneDirection::Left)),
            KeyCode::Right => Some(TmuxPrefixCommand::ResizePane(PaneDirection::Right)),
            KeyCode::Up => Some(TmuxPrefixCommand::ResizePane(PaneDirection::Up)),
            KeyCode::Down => Some(TmuxPrefixCommand::ResizePane(PaneDirection::Down)),
            _ => None,
        };
    }
    match key.code {
        KeyCode::Char('n') => Some(TmuxPrefixCommand::NextWindow),
        KeyCode::Char('p') => Some(TmuxPrefixCommand::PreviousWindow),
        KeyCode::Char('c') => Some(TmuxPrefixCommand::CreateWindow),
        KeyCode::Char('&') => Some(TmuxPrefixCommand::KillWindow),
        KeyCode::Char('f') => Some(TmuxPrefixCommand::FindWindow),
        KeyCode::Char('x') => Some(TmuxPrefixCommand::KillPane),
        KeyCode::Char('$') => Some(TmuxPrefixCommand::RenameSession),
        KeyCode::Char(',') => Some(TmuxPrefixCommand::RenameWindow),
        KeyCode::Char('w') => Some(TmuxPrefixCommand::ListWindows),
        KeyCode::Char('l') => Some(TmuxPrefixCommand::LastWindow),
        KeyCode::Char('%') => Some(TmuxPrefixCommand::SplitHorizontal),
        KeyCode::Char('"') => Some(TmuxPrefixCommand::SplitVertical),
        KeyCode::Char('o') => Some(TmuxPrefixCommand::NextPane),
        KeyCode::Char(';') => Some(TmuxPrefixCommand::LastPane),
        KeyCode::Char('q') => Some(TmuxPrefixCommand::DisplayPanes),
        KeyCode::Left => Some(TmuxPrefixCommand::SelectPane(PaneDirection::Left)),
        KeyCode::Right => Some(TmuxPrefixCommand::SelectPane(PaneDirection::Right)),
        KeyCode::Up => Some(TmuxPrefixCommand::SelectPane(PaneDirection::Up)),
        KeyCode::Down => Some(TmuxPrefixCommand::SelectPane(PaneDirection::Down)),
        KeyCode::Char('{') => Some(TmuxPrefixCommand::SwapPaneUp),
        KeyCode::Char('}') => Some(TmuxPrefixCommand::SwapPaneDown),
        KeyCode::Char('z') => Some(TmuxPrefixCommand::TogglePaneZoom),
        KeyCode::Char('[') => Some(TmuxPrefixCommand::CopyMode),
        KeyCode::Char(']') => Some(TmuxPrefixCommand::PasteBuffer),
        KeyCode::Char(':') => Some(TmuxPrefixCommand::CommandPrompt),
        KeyCode::Char('?') => Some(TmuxPrefixCommand::Help),
        KeyCode::Char(ch) if ch.is_ascii_digit() => {
            ch.to_digit(10).map(TmuxPrefixCommand::SelectWindow)
        }
        _ => None,
    }
}

pub(crate) fn tmux_prefix_bytes() -> Vec<u8> {
    vec![0x02]
}

pub(crate) fn key_event_bytes(key: &KeyEvent) -> Option<Vec<u8>> {
    if !is_key_press(key) {
        return None;
    }
    match &key.code {
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
        KeyCode::Char(ch) => char_key_bytes(*ch, key.modifiers),
        KeyCode::F(number) => function_key_bytes(*number),
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
