use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub(crate) use crate::terminal_key::key_to_bytes;
use crate::{
    region_types::{ScreenRegionAxis, ScreenRegionFocus},
    terminal_key::SCREEN_CONTROL_PREFIX,
};

mod action;
pub(crate) use self::action::ScreenInputAction;

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
    if (key.code == KeyCode::Char('[') || key.code == KeyCode::Esc)
        && key.modifiers.is_empty()
    {
        return Some(ScreenInputAction::CopyMode);
    }

        match key.code {
            KeyCode::Char('r') | KeyCode::Char('R') if key.modifiers.is_empty() => {
                Some(ScreenInputAction::WrapToggle)
            }
            KeyCode::Char('F')
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                Some(ScreenInputAction::Fit)
            }
            KeyCode::Char('Z')
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                Some(ScreenInputAction::Reset)
            }
            KeyCode::Char('l') | KeyCode::Char('L')
                if key.modifiers.is_empty() || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Some(ScreenInputAction::Redisplay)
            }
            KeyCode::Char('_')
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                Some(ScreenInputAction::SilenceToggle)
            }
            KeyCode::Char('M')
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                Some(ScreenInputAction::MonitorToggle)
            }
            KeyCode::Char('m') | KeyCode::Char('M')
                if key.modifiers.is_empty() || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Some(ScreenInputAction::LastMessage)
            }
            KeyCode::Char('C')
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                Some(ScreenInputAction::Clear)
            }
            KeyCode::Char('c') | KeyCode::Char('C')
                if key.modifiers.is_empty() || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Some(ScreenInputAction::NewWindow)
            }
            KeyCode::Char('D') if key.modifiers == KeyModifiers::SHIFT => {
                Some(ScreenInputAction::DetachAll)
            }
            KeyCode::Char('d') | KeyCode::Char('D')
                if key.modifiers.is_empty() || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Some(ScreenInputAction::Detach)
            }
            KeyCode::Char('k') | KeyCode::Char('K')
                if key.modifiers.is_empty() || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Some(ScreenInputAction::Kill)
            }
            KeyCode::Char('N') if key.modifiers == KeyModifiers::SHIFT => {
                Some(ScreenInputAction::Number)
            }
            KeyCode::Char('n') | KeyCode::Char('N')
                if key.modifiers.is_empty() || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Some(ScreenInputAction::NextWindow)
            }
            KeyCode::Char(' ') if key.modifiers.is_empty() => Some(ScreenInputAction::NextWindow),
            KeyCode::Char('p') | KeyCode::Char('P')
                if key.modifiers.is_empty() || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Some(ScreenInputAction::PreviousWindow)
            }
            KeyCode::Char('S') if key.modifiers == KeyModifiers::SHIFT => {
                Some(ScreenInputAction::SplitRegion(ScreenRegionAxis::Horizontal))
            }
            KeyCode::Char('|')
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                Some(ScreenInputAction::SplitRegion(ScreenRegionAxis::Vertical))
            }
            KeyCode::Tab if key.modifiers.is_empty() => {
                Some(ScreenInputAction::FocusRegion(ScreenRegionFocus::Next))
            }
            KeyCode::Char('X') if key.modifiers == KeyModifiers::SHIFT => {
                Some(ScreenInputAction::RemoveRegion)
            }
            KeyCode::Char('Q') if key.modifiers == KeyModifiers::SHIFT => {
                Some(ScreenInputAction::OnlyRegion)
            }
            KeyCode::Char('q') | KeyCode::Char('Q')
                if key.modifiers.is_empty() || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Some(ScreenInputAction::Bytes(vec![0x11]))
            }
            KeyCode::Char('s') | KeyCode::Char('S')
                if key.modifiers.is_empty() || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Some(ScreenInputAction::Bytes(vec![0x13]))
            }
            KeyCode::Backspace if key.modifiers.is_empty() => Some(ScreenInputAction::PreviousWindow),
            KeyCode::Char('h') | KeyCode::Char('H')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Some(ScreenInputAction::PreviousWindow)
            }
            KeyCode::Char('H') if key.modifiers == KeyModifiers::SHIFT => {
                Some(ScreenInputAction::LogToggle)
            }
            KeyCode::Char('h') | KeyCode::Char('H') if key.modifiers.is_empty() => {
                Some(ScreenInputAction::Hardcopy)
            }
            KeyCode::Char('i') | KeyCode::Char('I')
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::CONTROL =>
            {
                Some(ScreenInputAction::Info)
            }
            KeyCode::Char('t') | KeyCode::Char('T')
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::CONTROL =>
            {
                Some(ScreenInputAction::Time)
            }
            KeyCode::Char('v') | KeyCode::Char('V') if key.modifiers.is_empty() => {
                Some(ScreenInputAction::Version)
            }
            KeyCode::Char('W') if key.modifiers == KeyModifiers::SHIFT => {
                Some(ScreenInputAction::WidthToggle)
            }
            KeyCode::Char('w') | KeyCode::Char('W')
                if key.modifiers.is_empty() || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Some(ScreenInputAction::Windows)
            }
            KeyCode::Char(':')
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                Some(ScreenInputAction::CommandPrompt)
            }
            KeyCode::Char('\'') if key.modifiers.is_empty() => {
                Some(ScreenInputAction::SelectPrompt)
            }            KeyCode::Char(c) if key.modifiers.is_empty() && c.is_ascii_digit() => {
                Some(ScreenInputAction::SelectWindow(c.to_digit(10).unwrap_or(0) as usize))
            }
            KeyCode::Char('*') if key.modifiers.is_empty() => Some(ScreenInputAction::Displays),
            KeyCode::Char('?') if key.modifiers.is_empty() => Some(ScreenInputAction::Help),
            KeyCode::Char('.') if key.modifiers.is_empty() => Some(ScreenInputAction::DumpTermcap),
            KeyCode::Char(',') if key.modifiers.is_empty() => Some(ScreenInputAction::License),
            KeyCode::Char('<') if key.modifiers.is_empty() => Some(ScreenInputAction::ReadBuffer),
            KeyCode::Char('>') if key.modifiers.is_empty() => Some(ScreenInputAction::WriteBuffer),
            KeyCode::Char('=') if key.modifiers.is_empty() => Some(ScreenInputAction::RemoveBuffer),
            KeyCode::Char('\\') if key.modifiers.is_empty() => Some(ScreenInputAction::Quit),
            KeyCode::Char('"') if key.modifiers.is_empty() => Some(ScreenInputAction::Windows),
            KeyCode::Char(']')
                if key.modifiers.is_empty() || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Some(ScreenInputAction::Paste)
            }
            KeyCode::Char('A')
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                Some(ScreenInputAction::Title)
            }
            KeyCode::Char('a') if key.modifiers.is_empty() => {
                Some(ScreenInputAction::Bytes(vec![SCREEN_CONTROL_PREFIX]))
            }
            _ if is_screen_prefix_key(key) => Some(ScreenInputAction::LastWindow),
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

fn is_screen_prefix_key(key: KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('a') | KeyCode::Char('A'))
        && key.modifiers.contains(KeyModifiers::CONTROL)
}

#[cfg(test)]
#[path = "terminal_input_tests.rs"]
mod tests;
