use std::collections::HashSet;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    signal_menu::{self, SignalMenuState},
    footer::footer_line,
    io_view::draw_io,
    format::{format_bytes, format_duration},
    meter::meter_line,
    network_view::draw_network,
    overview_view::draw_overview,
    processes_view::draw_processes,
    model::{Snapshot, SortMode},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Tab {
    Overview,
    Processes,
    Io,
    Network,
}

impl Tab {
    pub(crate) fn next(self) -> Self {
        match self {
            Self::Overview => Self::Processes,
            Self::Processes => Self::Io,
            Self::Io => Self::Network,
            Self::Network => Self::Overview,
        }
    }

    pub(crate) fn previous(self) -> Self {
        match self {
            Self::Overview => Self::Network,
            Self::Processes => Self::Overview,
            Self::Io => Self::Processes,
            Self::Network => Self::Io,
        }
    }
}

pub(crate) fn draw(
    frame: &mut Frame<'_>,
    snapshot: &Snapshot,
    tab: Tab,
    sort: SortMode,
    sort_inverted: bool,
    tree: bool,
    selected: usize,
    filter: &str,
    filtering: bool,
    search: &str,
    searching: bool,
    detail_scroll: usize,
    io_scroll: usize,
    network_scroll: usize,
    refresh_ms: u64,
    followed_pid: Option<&str>,
    signal_state: Option<&SignalMenuState>,
    tagged_pids: &HashSet<String>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());
    draw_header(frame, chunks[0], snapshot, tab, followed_pid);
    match tab {
        Tab::Overview => draw_overview(frame, chunks[1], snapshot, sort, selected, tagged_pids),
        Tab::Processes => draw_processes(frame, chunks[1], snapshot, sort, tree, selected, filter, detail_scroll, tagged_pids),
        Tab::Io => draw_io(frame, chunks[1], snapshot, sort, io_scroll, selected),
        Tab::Network => draw_network(frame, chunks[1], snapshot, sort, network_scroll, selected),
    }
    frame.render_widget(Paragraph::new(footer_line(
        sort,
        sort_inverted,
        tree,
        filter,
        filtering,
        search,
        searching,
        refresh_ms,
        signal_state.map(SignalMenuState::pid),
    )), chunks[2]);
    if let Some(state) = signal_state {
        signal_menu::draw(frame, state);
    }
}

fn draw_header(
    frame: &mut Frame<'_>,
    area: Rect,
    snapshot: &Snapshot,
    tab: Tab,
    followed_pid: Option<&str>,
) {
    let cpu = snapshot.cpu_usage;
    let mem = format_bytes(snapshot.used_memory);
    let total = format_bytes(snapshot.total_memory);
    let swap = format_bytes(snapshot.used_swap);
    let swap_total = format_bytes(snapshot.total_swap);
    let lines = vec![
        meter_line("CPU", cpu as f64, 100.0, 16, format!("{cpu:>5.1}%  Tasks:{} shown/{} total  Load:{:.2} {:.2} {:.2}", snapshot.filtered_process_count, snapshot.process_count, snapshot.load_average.one, snapshot.load_average.five, snapshot.load_average.fifteen)),
        meter_line("MEM", snapshot.used_memory as f64, snapshot.total_memory as f64, 16, format!("{mem}/{total}  Uptime:{}", format_duration(snapshot.uptime))),
        meter_line("SWP", snapshot.used_swap as f64, snapshot.total_swap as f64, 16, format!("{swap}/{swap_total}")),
        tab_line(tab),
        status_line(followed_pid),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}

fn status_line(followed_pid: Option<&str>) -> Line<'static> {
    let mut spans = Vec::new();
    if let Some(pid) = followed_pid {
        spans.push(Span::styled(
            format!(" {} | ", terman_common::builtin_htop_follow_status_hint(pid)),
            Style::default().fg(Color::Black).bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    }
    spans.push(Span::styled(
        terman_common::builtin_htop_help_hint(),
        Style::default().fg(Color::DarkGray),
    ));
    Line::from(spans)
}

fn tab_line(active: Tab) -> Line<'static> {
    Line::from(vec![
        tab_span(active, Tab::Overview, terman_common::builtin_htop_tab_overview_hint()),
        Span::raw(" "),
        tab_span(active, Tab::Processes, terman_common::builtin_htop_tab_processes_hint()),
        Span::raw(" "),
        tab_span(active, Tab::Io, terman_common::builtin_htop_tab_io_hint()),
        Span::raw(" "),
        tab_span(active, Tab::Network, terman_common::builtin_htop_tab_network_hint()),
    ])
}

fn tab_span(active: Tab, tab: Tab, label: String) -> Span<'static> {
    let style = if active == tab {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    Span::styled(format!(" {label} "), style)
}
