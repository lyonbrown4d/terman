use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{
    app_events::{
        confirm_mouse_kill, handle_filter_input, handle_help_input, handle_kill_input,
        handle_search_input, handle_sort_menu_input, selected_process_pid,
    },
    app_input::{
        adjust_refresh, clamp_selection, delay_key, filter_key, help_key, interrupt_key, kill_key,
        move_selection, navigation_key, next_tab, quit_key, search_key, sort_key, tree_key,
    },
    app_terminal::TerminalGuard,
    cli::HtopArgs,
    help,
    interrupt::InterruptFlag,
    metrics::Metrics,
    model::{IoRow, ProcessRow, SocketRow, SortMode},
    mouse::{self, MouseContext},
    render::{self, Tab},
    selected_scroll::{keep_selected_visible, selected_data_index},
    sort_menu,
    tab_sort::normalize_sort_for_tab,
};

pub async fn run(args: HtopArgs) -> Result<(), Box<dyn Error>> {
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut metrics = Metrics::new();
    let mut tab = Tab::Overview;
    let mut sort = SortMode::Cpu;
    let mut sort_menu_open = false;
    let mut sort_cursor = sort;
    let mut tree = false;
    let mut help_open = false;
    let mut kill_target: Option<String> = None;
    let mut selected = 0usize;
    let mut detail_scroll = 0usize;
    let mut io_scroll = 0usize;
    let mut network_scroll = 0usize;
    let mut visibility_anchor: Option<(Tab, SortMode, Option<String>, Option<usize>, Option<(u16, u16)>)> = None;
    let mut refresh_ms = args.refresh_ms.max(100);
    let mut filter = String::new();
    let mut filter_input: Option<String> = None;
    let mut search = String::new();
    let mut search_input: Option<String> = None;
    let interrupt = InterruptFlag::new();
    interrupt.listen_for_ctrl_c();

    loop {
        metrics.refresh();
        let active_filter = filter_input.as_deref().unwrap_or(&filter);
        let active_search = search_input.as_deref().unwrap_or(&search);
        sort = normalize_sort_for_tab(tab, sort);
        let snapshot = metrics.snapshot(sort, active_filter, tree);
        selected = clamp_selection(selected, snapshot.processes.len());
        io_scroll = io_scroll.min(snapshot.io.len().saturating_sub(1));
        network_scroll = network_scroll
            .min(snapshot.networks.len().max(snapshot.sockets.len()).saturating_sub(1));
        let next_visibility_anchor = (
            tab,
            sort,
            snapshot.processes.get(selected).map(|row| row.pid.clone()),
            selected_data_index(tab, &snapshot, selected),
            terman_common::current_terminal_size().ok(),
        );
        if visibility_anchor.as_ref() != Some(&next_visibility_anchor) {
            keep_selected_visible(tab, &snapshot, selected, &mut io_scroll, &mut network_scroll);
            visibility_anchor = Some(next_visibility_anchor);
        }
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
                    detail_scroll,
                    io_scroll,
                    network_scroll,
                    refresh_ms,
                    kill_target.as_deref(),
                );
                if sort_menu_open {
                    sort_menu::draw(frame, sort_cursor);
                }
            }
        })?;
        if args.once {
            return Ok(());
        }
        if poll_until_refresh(
            &interrupt,
            &mut metrics,
            &mut refresh_ms,
            &mut tab,
            &mut sort,
            &mut sort_menu_open,
            &mut sort_cursor,
            &mut tree,
            &mut help_open,
            &mut kill_target,
            &mut selected,
            &mut detail_scroll,
            &mut io_scroll,
            &mut network_scroll,
            snapshot.processes.as_slice(),
            snapshot.io.as_slice(),
            snapshot.sockets.as_slice(),
            snapshot.cpu_cores.len(),
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
    interrupt: &InterruptFlag,
    metrics: &mut Metrics,
    refresh_ms: &mut u64,
    tab: &mut Tab,
    sort: &mut SortMode,
    sort_menu_open: &mut bool,
    sort_cursor: &mut SortMode,
    tree: &mut bool,
    help_open: &mut bool,
    kill_target: &mut Option<String>,
    selected: &mut usize,
    detail_scroll: &mut usize,
    io_scroll: &mut usize,
    network_scroll: &mut usize,
    processes: &[ProcessRow],
    io: &[IoRow],
    sockets: &[SocketRow],
    cpu_core_count: usize,
    filter: &mut String,
    filter_input: &mut Option<String>,
    search: &mut String,
    search_input: &mut Option<String>,
) -> io::Result<bool> {
    if interrupt.interrupted() {
        return Ok(true);
    }
    let refresh = Duration::from_millis((*refresh_ms).max(100));
    let deadline = Instant::now() + refresh;
    while Instant::now() < deadline {
        if interrupt.interrupted() {
            return Ok(true);
        }
        if event::poll(Duration::from_millis(50))? {
            let redraw = match event::read()? {
                Event::Key(key) if interrupt_key(&key) => return Ok(true),
                Event::Key(key) if key.kind == KeyEventKind::Release => false,
                Event::Key(key) if key.code == KeyCode::F(10) => return Ok(true),
                Event::Mouse(mouse_event) => {
                    let action = mouse::handle_mouse(mouse_event, MouseContext {
                        tab,
                        sort,
                        sort_menu_open,
                        sort_cursor,
                        tree,
                        help_open,
                        selected,
                        detail_scroll,
                        io_scroll,
                        network_scroll,
                        processes,
                        io,
                        sockets,
                        cpu_core_count,
                        filter: filter_input.as_deref().unwrap_or(filter.as_str()),
                        search: search_input.as_deref().unwrap_or(search.as_str()),
                        kill_target: kill_target.as_deref(),
                        refresh_ms: *refresh_ms,
                    });
                    match action {
                        mouse::MouseAction::Quit => return Ok(true),
                        mouse::MouseAction::Search => *search_input = Some(search.clone()),
                        mouse::MouseAction::Filter => *filter_input = Some(filter.clone()),
                        mouse::MouseAction::Kill => {
                            *kill_target = selected_process_pid(processes, *selected);
                        }
                        mouse::MouseAction::ConfirmKill => confirm_mouse_kill(metrics, kill_target),
                        mouse::MouseAction::CancelKill => *kill_target = None,
                        mouse::MouseAction::DelayFaster => {
                            adjust_refresh(refresh_ms, KeyCode::Char('+'));
                        }
                        mouse::MouseAction::DelaySlower => {
                            adjust_refresh(refresh_ms, KeyCode::Char('-'));
                        }
                        _ => {}
                    }
                    action != mouse::MouseAction::Ignored
                }
                Event::Key(key) if handle_kill_input(key.code, metrics, kill_target) => true,
                Event::Key(key) if handle_help_input(key.code, help_open) => true,
                Event::Key(key) if handle_sort_menu_input(key.code, sort, sort_menu_open, sort_cursor) => true,
                Event::Key(key) if handle_search_input(key.code, search, search_input, selected, processes) => true,
                Event::Key(key) if handle_filter_input(key.code, filter, filter_input) => true,
                Event::Key(key) if key.code == KeyCode::Esc && !filter.is_empty() => {
                    filter.clear();
                    true
                }
                Event::Key(key) if quit_key(key.code) => return Ok(true),
                Event::Key(key) if navigation_key(key.code) => {
                    let next = move_selection(*selected, processes.len(), key.code);
                    if next != *selected {
                        *detail_scroll = 0;
                    }
                    *selected = next;
                    true
                }
                Event::Key(key) if delay_key(key.code) => {
                    adjust_refresh(refresh_ms, key.code);
                    true
                }
                Event::Key(key) if kill_key(key.code) => {
                    *kill_target = selected_process_pid(processes, *selected);
                    true
                }
                Event::Key(key) if help_key(key.code) => {
                    *help_open = true;
                    true
                }
                Event::Key(key) if search_key(key.code) => {
                    *search_input = Some(search.clone());
                    true
                }
                Event::Key(key) if filter_key(key.code) => {
                    *filter_input = Some(filter.clone());
                    true
                }
                Event::Key(key) if sort_key(key.code) => {
                    *sort_cursor = *sort;
                    *sort_menu_open = true;
                    true
                }
                Event::Key(key) if tree_key(key.code) => {
                    *tree = !*tree;
                    true
                }
                Event::Key(key) => {
                    let next = next_tab(*tab, &key);
                    let changed = next != *tab;
                    *tab = next;
                    changed
                }
                Event::Resize(_, _) => true,
                _ => false,
            };
            if redraw {
                return Ok(false);
            }
        }
    }
    Ok(false)
}