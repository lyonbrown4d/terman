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
    help,
    metrics::Metrics, model::{ProcessRow, SortMode},
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
    let mut tree = false;
    let mut help_open = false;
    let mut kill_target: Option<String> = None;
    let mut selected = 0usize;
    let mut refresh_ms = args.refresh_ms.max(100);
    let mut filter = String::new();
    let mut filter_input: Option<String> = None;
    let mut search = String::new();
    let mut search_input: Option<String> = None;

    loop {
        metrics.refresh();
        let active_filter = filter_input.as_deref().unwrap_or(&filter);
        let active_search = search_input.as_deref().unwrap_or(&search);
        let snapshot = metrics.snapshot(sort, active_filter, tree);
        selected = clamp_selection(selected, snapshot.processes.len());
        terminal.draw(|frame| {
            if help_open {
                help::draw(frame);
            } else {
                render::draw(
                    frame,
                    &snapshot,
                    tab,
                    sort,
                    tree,
                    selected,
                    active_filter,
                    filter_input.is_some(),
                    active_search,
                    search_input.is_some(),
                    refresh_ms,
                    kill_target.as_deref(),
                );
            }
        })?;
        if args.once {
            return Ok(());
        }
        if poll_until_refresh(
            &mut metrics,
            &mut refresh_ms,
            &mut tab,
            &mut sort,
            &mut tree,
            &mut help_open,
            &mut kill_target,
            &mut selected,
            snapshot.processes.as_slice(),
            &mut filter,
            &mut filter_input,
            &mut search,
            &mut search_input,
        )? {
            return Ok(());
        }
    }
}

fn poll_until_refresh(
    metrics: &mut Metrics,
    refresh_ms: &mut u64,
    tab: &mut Tab,
    sort: &mut SortMode,
    tree: &mut bool,
    help_open: &mut bool,
    kill_target: &mut Option<String>,
    selected: &mut usize,
    processes: &[ProcessRow],
    filter: &mut String,
    filter_input: &mut Option<String>,
    search: &mut String,
    search_input: &mut Option<String>,
) -> io::Result<bool> {
    let refresh = Duration::from_millis((*refresh_ms).max(100));
    let deadline = Instant::now() + refresh;
    while Instant::now() < deadline {
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) if key.code == KeyCode::F(10) => return Ok(true),
                Event::Key(key) if handle_kill_input(key.code, metrics, kill_target) => {}
                Event::Key(key) if handle_help_input(key.code, help_open) => {}
                Event::Key(key) if handle_search_input(key.code, search, search_input, selected, processes) => {}
                Event::Key(key) if handle_filter_input(key.code, filter, filter_input) => {}
                Event::Key(key) if key.code == KeyCode::Esc && !filter.is_empty() => filter.clear(),
                Event::Key(key) if quit_key(key.code) => return Ok(true),
                Event::Key(key) if navigation_key(key.code) => *selected = move_selection(*selected, processes.len(), key.code),
                Event::Key(key) if delay_key(key.code) => adjust_refresh(refresh_ms, key.code),
                Event::Key(key) if kill_key(key.code) => *kill_target = selected_process_pid(processes, *selected),
                Event::Key(key) if help_key(key.code) => *help_open = true,
                Event::Key(key) if search_key(key.code) => *search_input = Some(search.clone()),
                Event::Key(key) if filter_key(key.code) => *filter_input = Some(filter.clone()),
                Event::Key(key) if sort_key(key.code) => *sort = sort.next(),
                Event::Key(key) if tree_key(key.code) => *tree = !*tree,
                Event::Key(key) => *tab = next_tab(*tab, key.code),
                Event::Resize(_, _) => break,
                _ => {}
            }
        }
    }
    Ok(false)
}

fn handle_kill_input(code: KeyCode, metrics: &mut Metrics, kill_target: &mut Option<String>) -> bool {
    let Some(pid) = kill_target.clone() else { return false; };
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => { let _ = metrics.kill_process(pid.as_str()); *kill_target = None; }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => *kill_target = None,
        _ => {}
    }
    true
}

fn selected_process_pid(processes: &[ProcessRow], selected: usize) -> Option<String> {
    processes.get(selected).map(|row| row.pid.clone())
}
fn handle_help_input(code: KeyCode, help_open: &mut bool) -> bool {
    if !*help_open { return false; }
    if matches!(code, KeyCode::Esc | KeyCode::F(1)) { *help_open = false; }
    true
}

fn handle_search_input(code: KeyCode, search: &mut String, search_input: &mut Option<String>, selected: &mut usize, processes: &[ProcessRow]) -> bool {
    let Some(input) = search_input.as_mut() else { return false; };
    match code {
        KeyCode::Enter => { *search = input.trim().to_string(); *selected = find_next(*selected, processes, search.as_str()); *search_input = None; }
        KeyCode::Esc => *search_input = None,
        KeyCode::Backspace => { input.pop(); }
        KeyCode::Char(ch) => input.push(ch),
        _ => {}
    }
    true
}

fn handle_filter_input(code: KeyCode, filter: &mut String, filter_input: &mut Option<String>) -> bool {
    let Some(input) = filter_input.as_mut() else { return false; };
    match code {
        KeyCode::Enter => { *filter = input.trim().to_string(); *filter_input = None; }
        KeyCode::Esc => *filter_input = None,
        KeyCode::Backspace => { input.pop(); }
        KeyCode::Char(ch) => input.push(ch),
        _ => {}
    }
    true
}

fn find_next(selected: usize, processes: &[ProcessRow], term: &str) -> usize {
    let term = term.trim().to_lowercase();
    if term.is_empty() || processes.is_empty() { return selected; }
    for offset in 1..=processes.len() {
        let index = (selected + offset) % processes.len();
        if process_matches_search(&processes[index], term.as_str()) { return index; }
    }
    selected
}

fn process_matches_search(row: &ProcessRow, term: &str) -> bool {
    row.pid.contains(term) || row.name.to_lowercase().contains(term) || row.command.to_lowercase().contains(term)
}

fn adjust_refresh(refresh_ms: &mut u64, code: KeyCode) {
    match code {
        KeyCode::Char('+') | KeyCode::Char('=') => *refresh_ms = refresh_ms.saturating_sub(100).max(100),
        KeyCode::Char('-') => *refresh_ms = (*refresh_ms + 100).min(60_000),
        _ => {}
    }
}

fn quit_key(code: KeyCode) -> bool { matches!(code, KeyCode::Char('q') | KeyCode::Esc | KeyCode::F(10)) }
fn help_key(code: KeyCode) -> bool { matches!(code, KeyCode::F(1) | KeyCode::Char('h')) }
fn search_key(code: KeyCode) -> bool { matches!(code, KeyCode::F(3)) }
fn filter_key(code: KeyCode) -> bool { matches!(code, KeyCode::Char('/') | KeyCode::F(4)) }
fn sort_key(code: KeyCode) -> bool { matches!(code, KeyCode::Char('s') | KeyCode::F(6)) }
fn tree_key(code: KeyCode) -> bool { matches!(code, KeyCode::Char('t') | KeyCode::F(5)) }
fn delay_key(code: KeyCode) -> bool { matches!(code, KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::Char('-')) }
fn kill_key(code: KeyCode) -> bool { matches!(code, KeyCode::F(9)) }
fn navigation_key(code: KeyCode) -> bool { matches!(code, KeyCode::Up | KeyCode::Down | KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End) }

fn move_selection(selected: usize, count: usize, code: KeyCode) -> usize {
    if count == 0 { return 0; }
    match code {
        KeyCode::Up => selected.saturating_sub(1),
        KeyCode::Down => (selected + 1).min(count - 1),
        KeyCode::PageUp => selected.saturating_sub(10),
        KeyCode::PageDown => (selected + 10).min(count - 1),
        KeyCode::Home => 0,
        KeyCode::End => count - 1,
        _ => selected,
    }
}

fn clamp_selection(selected: usize, count: usize) -> usize {
    if count == 0 { 0 } else { selected.min(count - 1) }
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
