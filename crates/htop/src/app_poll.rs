use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};

use crate::{
    app_events::{
        confirm_mouse_signal, handle_filter_input, handle_help_input, handle_search_input,
        handle_signal_input, handle_sort_menu_input, selected_process_pid,
    },
    app_input::{
        adjust_refresh, delay_key, filter_key, help_key, interrupt_key, invert_sort_key, kill_key,
        move_selection, navigation_key, next_tab, priority_delta, quit_key, search_key, sort_key,
        tree_key,
    },
    interrupt::InterruptFlag,
    metrics::Metrics,
    model::{IoRow, ProcessRow, SocketRow, SortMode},
    mouse::{self, MouseContext},
    render::Tab,
    signal_menu::SignalMenuState,
};

#[allow(clippy::too_many_arguments)]
pub(crate) fn poll_until_refresh(
    interrupt: &InterruptFlag,
    metrics: &mut Metrics,
    refresh_ms: &mut u64,
    tab: &mut Tab,
    sort: &mut SortMode,
    sort_inverted: &mut bool,
    sort_menu_open: &mut bool,
    sort_cursor: &mut SortMode,
    sort_header_pressed: &mut Option<SortMode>,
    tree: &mut bool,
    help_open: &mut bool,
    signal_menu: &mut Option<SignalMenuState>,
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
                    let action = mouse::handle_mouse(
                        mouse_event,
                        MouseContext {
                            tab,
                            sort,
                            sort_inverted,
                            sort_menu_open,
                            sort_cursor,
                            sort_header_pressed,
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
                            signal_menu,
                            refresh_ms: *refresh_ms,
                        },
                    );
                    match action {
                        mouse::MouseAction::Quit => return Ok(true),
                        mouse::MouseAction::Search => *search_input = Some(search.clone()),
                        mouse::MouseAction::Filter => *filter_input = Some(filter.clone()),
                        mouse::MouseAction::Kill => {
                            *signal_menu = selected_process_pid(processes, *selected)
                                .map(SignalMenuState::new);
                        }
                        mouse::MouseAction::ConfirmKill => {
                            confirm_mouse_signal(metrics, signal_menu);
                        }
                        mouse::MouseAction::CancelKill => *signal_menu = None,
                        mouse::MouseAction::PriorityHigher => {
                            adjust_selected_priority(metrics, processes, *selected, -1);
                        }
                        mouse::MouseAction::PriorityLower => {
                            adjust_selected_priority(metrics, processes, *selected, 1);
                        }
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
                Event::Key(key) if handle_signal_input(key.code, metrics, signal_menu) => true,
                Event::Key(key) if handle_help_input(key.code, help_open) => true,
                Event::Key(key)
                    if handle_sort_menu_input(
                        key.code,
                        sort,
                        sort_menu_open,
                        sort_cursor,
                    ) =>
                {
                    true
                }
                Event::Key(key)
                    if handle_search_input(
                        key.code,
                        search,
                        search_input,
                        selected,
                        processes,
                    ) =>
                {
                    true
                }
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
                    *signal_menu = selected_process_pid(processes, *selected)
                        .map(SignalMenuState::new);
                    true
                }
                Event::Key(key) if priority_delta(key.code).is_some() => {
                    adjust_selected_priority(
                        metrics,
                        processes,
                        *selected,
                        priority_delta(key.code).unwrap_or_default(),
                    );
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
                Event::Key(key) if invert_sort_key(key.code) => {
                    *sort_inverted = !*sort_inverted;
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

fn adjust_selected_priority(
    metrics: &mut Metrics,
    processes: &[ProcessRow],
    selected: usize,
    delta: i32,
) {
    if let Some(pid) = selected_process_pid(processes, selected) {
        let _ = metrics.adjust_process_priority(&pid, delta);
    }
}