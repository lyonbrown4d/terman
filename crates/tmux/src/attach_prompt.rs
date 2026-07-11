use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(crate) enum PromptAction {
    Editing,
    Cancel,
    Execute(String),
}

pub(crate) fn edit_prompt(
    key: &KeyEvent,
    input: &mut String,
) -> PromptAction {
    match key.code {
        KeyCode::Enter => PromptAction::Execute(std::mem::take(input)),
        KeyCode::Esc => PromptAction::Cancel,
        KeyCode::Char('c' | 'C')
            if key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            PromptAction::Cancel
        }
        KeyCode::Backspace => {
            input.pop();
            PromptAction::Editing
        }
        KeyCode::Char('u' | 'U')
            if key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            input.clear();
            PromptAction::Editing
        }
        KeyCode::Char(ch)
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            input.push(ch);
            PromptAction::Editing
        }
        _ => PromptAction::Editing,
    }
}