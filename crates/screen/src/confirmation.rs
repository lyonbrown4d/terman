use std::io::{self, Write};

use crossterm::event::{
    self, Event, KeyCode, KeyEventKind, KeyModifiers,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScreenConfirmation {
    KillWindow,
    QuitSession,
}

pub(crate) fn prompt_screen_confirmation(
    confirmation: ScreenConfirmation,
) -> io::Result<bool> {
    let prompt = match confirmation {
        ScreenConfirmation::KillWindow => {
            terman_common::builtin_screen_confirm_kill_hint()
        }
        ScreenConfirmation::QuitSession => {
            terman_common::builtin_screen_confirm_quit_hint()
        }
    };
    let mut stdout = io::stdout();
    write!(stdout, "\r\n{prompt} ")?;
    stdout.flush()?;

    loop {
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        let accepted = match key.code {
            KeyCode::Char('y' | 'Y')
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                Some(true)
            }
            KeyCode::Char('n' | 'N') | KeyCode::Esc | KeyCode::Enter => {
                Some(false)
            }
            KeyCode::Char('c' | 'C')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                Some(false)
            }
            _ => None,
        };
        if let Some(accepted) = accepted {
            write!(stdout, "\r\n")?;
            stdout.flush()?;
            return Ok(accepted);
        }
    }
}