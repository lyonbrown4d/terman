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
    let mut filter = String::new();
    let mut filter_input: Option<String> = None;

    loop {
        metrics.refresh();
        let active_filter = filter_input.as_deref().unwrap_or(&filter);
        let snapshot = metrics.snapshot(sort, active_filter);
        terminal.draw(|frame| {
            render::draw(frame, &snapshot, tab, sort, active_filter, filter_input.is_some())
        })?;
        if args.once {
            return Ok(());
        }
        if poll_until_refresh(args.refresh_ms, &mut tab, &mut sort, &mut filter, &mut filter_input)? {
            return Ok(());
        }
    }
}

fn poll_until_refresh(
    refresh_ms: u64,
    tab: &mut Tab,
    sort: &mut SortMode,
    filter: &mut String,
    filter_input: &mut Option<String>,
) -> io::Result<bool> {
    let refresh = Duration::from_millis(refresh_ms.max(100));
    let deadline = Instant::now() + refresh;
    while Instant::now() < deadline {
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.code == KeyCode::F(10) => return Ok(true),
                Event::Key(key) if handle_filter_input(key.code, filter, filter_input) => {}
                Event::Key(key) if key.code == KeyCode::Esc && !filter.is_empty() => filter.clear(),
                Event::Key(key) if quit_key(key.code) => return Ok(true),
                Event::Key(key) if filter_key(key.code) => *filter_input = Some(filter.clone()),
                Event::Key(key) if sort_key(key.code) => *sort = sort.next(),
                Event::Key(key) => *tab = next_tab(*tab, key.code),
                Event::Resize(_, _) => break,
                _ => {}
            }
        }
    }
    Ok(false)
}

fn handle_filter_input(
    code: KeyCode,
    filter: &mut String,
    filter_input: &mut Option<String>,
) -> bool {
    let Some(input) = filter_input.as_mut() else {
        return false;
    };
    match code {
        KeyCode::Enter => {
            *filter = input.trim().to_string();
            *filter_input = None;
        }
        KeyCode::Esc => *filter_input = None,
        KeyCode::Backspace => {
            input.pop();
        }
        KeyCode::Char(ch) => input.push(ch),
        _ => {}
    }
    true
}

fn quit_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('q') | KeyCode::Esc | KeyCode::F(10))
}

fn filter_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('/') | KeyCode::F(4))
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
