use std::{error::Error, io, time::{Duration, Instant}};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use crate::{cli::HtopArgs, metrics::Metrics, render::{self, Tab}};

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

pub async fn run(args: HtopArgs) -> Result<(), Box<dyn Error>> {
    let _guard = TerminalGuard::enter()?;
    let mut stdout = io::stdout();
    let mut metrics = Metrics::new();
    let mut tab = Tab::Processes;

    loop {
        metrics.refresh();
        render::draw(&mut stdout, &metrics.snapshot(), tab)?;
        if args.once {
            return Ok(());
        }
        let refresh = Duration::from_millis(args.refresh_ms.max(100));
        let deadline = Instant::now() + refresh;
        while Instant::now() < deadline {
            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) if quit_key(key.code) => return Ok(()),
                    Event::Key(key) => tab = next_tab(tab, key.code),
                    Event::Resize(_, _) => break,
                    _ => {}
                }
            }
        }
    }
}

fn quit_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('q') | KeyCode::Esc)
}

fn next_tab(tab: Tab, code: KeyCode) -> Tab {
    match code {
        KeyCode::Tab | KeyCode::Right => tab.next(),
        KeyCode::Left => tab.previous(),
        KeyCode::Char('1') => Tab::Processes,
        KeyCode::Char('2') => Tab::Io,
        KeyCode::Char('3') => Tab::Network,
        _ => tab,
    }
}