use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

pub(crate) fn mouse_event_bytes(event: MouseEvent) -> Option<Vec<u8>> {
    let (code, suffix) = match event.kind {
        MouseEventKind::Down(button) => (button_code(button, event.modifiers)?, 'M'),
        MouseEventKind::Up(button) => (button_code(button, event.modifiers)?, 'm'),
        MouseEventKind::Drag(button) => (button_code(button, event.modifiers)? + 32, 'M'),
        MouseEventKind::Moved => (modifier_code(event.modifiers) + 35, 'M'),
        MouseEventKind::ScrollUp => (modifier_code(event.modifiers) + 64, 'M'),
        MouseEventKind::ScrollDown => (modifier_code(event.modifiers) + 65, 'M'),
        _ => return None,
    };
    Some(format!("\x1b[<{code};{};{}{suffix}", event.column.saturating_add(1), event.row.saturating_add(1)).into_bytes())
}

fn button_code(button: MouseButton, modifiers: KeyModifiers) -> Option<u16> {
    let base = match button {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
    };
    Some(base + modifier_code(modifiers))
}

fn modifier_code(modifiers: KeyModifiers) -> u16 {
    let mut code = 0;
    if modifiers.contains(KeyModifiers::SHIFT) { code += 4; }
    if modifiers.contains(KeyModifiers::ALT) { code += 8; }
    if modifiers.contains(KeyModifiers::CONTROL) { code += 16; }
    code
}

#[cfg(test)]
mod tests {
    use super::mouse_event_bytes;
    use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

    #[test]
    fn encodes_left_click_as_sgr_mouse_sequence() {
        let event = MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column: 4, row: 2, modifiers: KeyModifiers::empty() };
        assert_eq!(mouse_event_bytes(event), Some(b"\x1b[<0;5;3M".to_vec()));
    }
}