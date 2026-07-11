use std::io::{self, Write};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

pub(super) fn read_attach_prompt(prompt: &str) -> io::Result<Option<String>> {
    let mut stdout = io::stdout();
    write!(stdout, "\r\n{prompt} ")?;
    stdout.flush()?;

    let mut value = String::new();
    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Enter => return finish_prompt(&mut stdout, value),
                KeyCode::Esc => return cancel_prompt(&mut stdout),
                KeyCode::Char('c' | 'C')
                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    return cancel_prompt(&mut stdout);
                }
                KeyCode::Char('u' | 'U')
                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    clear_value(&mut stdout, &mut value)?;
                }
                KeyCode::Backspace => erase_last(&mut stdout, &mut value)?,
                KeyCode::Char(character) if printable(key.modifiers) => {
                    value.push(character);
                    write!(stdout, "{character}")?;
                    stdout.flush()?;
                }
                _ => {}
            }
        }
    }
}

fn finish_prompt(
    stdout: &mut io::Stdout,
    value: String,
) -> io::Result<Option<String>> {
    write!(stdout, "\r\n")?;
    stdout.flush()?;
    let value = value.trim().to_string();
    Ok((!value.is_empty()).then_some(value))
}

fn cancel_prompt(stdout: &mut io::Stdout) -> io::Result<Option<String>> {
    write!(stdout, "\r\n")?;
    stdout.flush()?;
    Ok(None)
}

fn clear_value(stdout: &mut io::Stdout, value: &mut String) -> io::Result<()> {
    while !value.is_empty() {
        erase_last(stdout, value)?;
    }
    Ok(())
}

fn erase_last(stdout: &mut io::Stdout, value: &mut String) -> io::Result<()> {
    let Some(character) = value.pop() else {
        return Ok(());
    };
    let width = terman_common::terminal_text_width(character.to_string().as_str()).max(1);
    let backspaces = "\u{8}".repeat(width as usize);
    write!(stdout, "{backspaces}{}{backspaces}", " ".repeat(width as usize))?;
    stdout.flush()
}

fn printable(modifiers: KeyModifiers) -> bool {
    !modifiers.contains(KeyModifiers::CONTROL)
        && !modifiers.contains(KeyModifiers::ALT)
}