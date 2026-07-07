use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{
    cli::HtopArgs,
    metrics::{Metrics, SortMode},
    render::{self, Tab},
};

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
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut metrics = Metrics::new();
    let mut tab = Tab::Overview;
    let mut sort = SortMode::Cpu;

    loop {
        metrics.refresh();
        terminal.draw(|frame| render::draw(frame, &metrics.snapshot(sort), tab, sort))?;
        if args.once {
            return Ok(());
        }
        let refresh = Duration::from_millis(args.refresh_ms.max(100));
        let deadline = Instant::now() + refresh;
        while Instant::now() < deadline {
            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) if quit_key(key.code) => return Ok(()),
                    Event::Key(key) if sort_key(key.code) => sort = sort.next(),
                    Event::Key(key) => tab = next_tab(tab, key.code),
                    Event::Resize(_, _) => break,
                    _ => {}
                }
            }
        }
    }
}

fn quit_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('q') | KeyCode::Esc | KeyCode::F(10))
}

fn sort_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('s') | KeyCode::F(6))
}

fn next_tab(tab: Tab, code: KeyCode) -> Tab {
    match code {
        KeyCode::Tab | KeyCode::Right => tab.next(),
        KeyCode::Left => tab.previous(),
        KeyCode::Char('1') => Tab::Overview,
        KeyCode::Char('2') => Tab::Processes,
        KeyCode::Char('3') => Tab::Io,
        KeyCode::Char('4') => Tab::Network,
        _ => tab,
    }
}
