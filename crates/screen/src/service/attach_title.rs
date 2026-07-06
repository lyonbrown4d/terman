use std::io::{self, Write};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use super::ipc_client::send_control_request;
use crate::ipc::{ScreenIpcEndpoint, ScreenIpcRequest};

pub(super) fn prompt_attach_title(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    let Some(title) = read_title()? else {
        return Ok(());
    };
    send_control_request(endpoint, ScreenIpcRequest::SetWindowTitle { title })
}

fn read_title() -> io::Result<Option<String>> {
    let mut stdout = io::stdout();
    write!(
        stdout,
        "\r\n{} ",
        terman_common::builtin_screen_attach_title_prompt_hint()
    )?;
    stdout.flush()?;

    let mut title = String::new();
    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Enter => return finish_title(&mut stdout, title),
                KeyCode::Esc => return cancel_title(&mut stdout),
                KeyCode::Backspace => pop_title_char(&mut stdout, &mut title)?,
                KeyCode::Char(c) if printable_title_char(key.modifiers) => {
                    title.push(c);
                    write!(stdout, "{c}")?;
                    stdout.flush()?;
                }
                _ => {}
            }
        }
    }
}

fn finish_title(stdout: &mut io::Stdout, title: String) -> io::Result<Option<String>> {
    write!(stdout, "\r\n")?;
    stdout.flush()?;
    let title = title.trim().to_string();
    if title.is_empty() {
        Ok(None)
    } else {
        Ok(Some(title))
    }
}

fn cancel_title(stdout: &mut io::Stdout) -> io::Result<Option<String>> {
    write!(stdout, "\r\n")?;
    stdout.flush()?;
    Ok(None)
}

fn pop_title_char(stdout: &mut io::Stdout, title: &mut String) -> io::Result<()> {
    if title.pop().is_some() {
        write!(stdout, "\u{8} \u{8}")?;
        stdout.flush()?;
    }
    Ok(())
}

fn printable_title_char(modifiers: KeyModifiers) -> bool {
    !modifiers.contains(KeyModifiers::CONTROL) && !modifiers.contains(KeyModifiers::ALT)
}