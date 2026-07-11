use std::io::{self, Write};

use crossterm::event::{
    KeyEvent, KeyEventKind, MouseEvent, MouseEventKind,
};

#[derive(Default)]
pub(crate) struct ScreenBlanker {
    active: bool,
}

impl ScreenBlanker {
    pub(crate) fn activate(&mut self) -> io::Result<()> {
        self.active = true;
        self.render()
    }

    pub(crate) fn is_active(&self) -> bool {
        self.active
    }

    pub(crate) fn dismiss_key(&mut self, key: &KeyEvent) -> io::Result<bool> {
        if !self.active || key.kind != KeyEventKind::Press {
            return Ok(false);
        }
        self.dismiss()?;
        Ok(true)
    }

    pub(crate) fn dismiss_mouse(&mut self, mouse: &MouseEvent) -> io::Result<bool> {
        if !self.active || !dismisses_blank(mouse.kind) {
            return Ok(false);
        }
        self.dismiss()?;
        Ok(true)
    }

    pub(crate) fn render(&self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }
        write_terminal(b"[?25l[0m[2J[H")
    }

    fn dismiss(&mut self) -> io::Result<()> {
        self.active = false;
        write_terminal(b"[?25h")
    }
}

fn dismisses_blank(kind: MouseEventKind) -> bool {
    matches!(
        kind,
        MouseEventKind::Down(_)
            | MouseEventKind::ScrollDown
            | MouseEventKind::ScrollUp
            | MouseEventKind::ScrollLeft
            | MouseEventKind::ScrollRight
    )
}

fn write_terminal(bytes: &[u8]) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    stdout.write_all(bytes)?;
    stdout.flush()
}