use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
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
    kill_target: Option<&str>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0), Constraint::Length(1)])
        .split(frame.area());
    draw_header(frame, chunks[0], snapshot, tab);
    match tab {
        Tab::Overview => draw_overview(frame, chunks[1], snapshot, selected),
        Tab::Processes => draw_processes(frame, chunks[1], snapshot, sort, tree, selected, filter, detail_scroll),
        Tab::Io => draw_io(frame, chunks[1], snapshot, io_scroll, selected),
        Tab::Network => draw_network(frame, chunks[1], snapshot, network_scroll, selected),
    }
    frame.render_widget(Paragraph::new(footer_line(sort, tree, filter, filtering, search, searching, refresh_ms, kill_target)), chunks[2]);
}

fn draw_header(frame: &mut Frame<'_>, area: Rect, snapshot: &Snapshot, tab: Tab) {
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
        Line::from(Span::styled(
            terman_common::builtin_htop_help_hint(),
            Style::default().fg(Color::DarkGray),
        )),
    ];
    frame.render_widget(Paragraph::new(lines), area);
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
