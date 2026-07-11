use std::collections::HashSet;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    command_display::ProcessCommandMode,
    core_meter::core_meter_lines,
    format::{format_bytes, format_duration},
    meter::meter_line,
    model::{Snapshot, SortMode},
    overview_layout,
    process_status::status_summary_line,
    process_table::{process_header_line, process_line},
};

pub(crate) fn draw_overview(
    frame: &mut Frame<'_>,
    area: Rect,
    snapshot: &Snapshot,
    sort: SortMode,
    command_mode: ProcessCommandMode,
    selected: usize,
    tagged_pids: &HashSet<String>,
) {
    let content_width = area.width.saturating_sub(2);
    let core_columns =
        overview_layout::core_columns(content_width, snapshot.cpu_cores.len());
    let core_rows = overview_layout::core_rows(
        area.height,
        content_width,
        snapshot.cpu_cores.len(),
    );
    let process_rows = overview_layout::process_rows(
        area.height,
        content_width,
        snapshot.cpu_cores.len(),
    );
    let process_start = overview_layout::visible_start(
        selected,
        process_rows,
        snapshot.processes.len(),
    );
    let mut lines = vec![
        meter_line(
            "CPU",
            snapshot.cpu_usage as f64,
            100.0,
            24,
            terman_common::builtin_htop_overview_cpu_hint(
                snapshot.cpu_usage,
                snapshot.cpu_count,
            ),
        ),
        plain_line(terman_common::builtin_htop_overview_host_hint(
            &snapshot.system.hostname,
            &snapshot.system.os,
        )),
        plain_line(terman_common::builtin_htop_overview_kernel_hint(
            &snapshot.system.kernel,
            &snapshot.system.arch,
        )),
        meter_line(
            "Mem",
            snapshot.used_memory as f64,
            snapshot.total_memory as f64,
            24,
            format!(
                "{} / {}",
                format_bytes(snapshot.used_memory),
                format_bytes(snapshot.total_memory)
            ),
        ),
        meter_line(
            "Swp",
            snapshot.used_swap as f64,
            snapshot.total_swap as f64,
            24,
            format!(
                "{} / {}",
                format_bytes(snapshot.used_swap),
                format_bytes(snapshot.total_swap)
            ),
        ),
        plain_line(terman_common::builtin_htop_overview_tasks_hint(
            snapshot.filtered_process_count,
            snapshot.process_count,
        )),
        status_summary_line(snapshot.processes.as_slice()),
        plain_line(terman_common::builtin_htop_overview_network_hint(
            &format_bytes(snapshot.received_per_refresh),
            &format_bytes(snapshot.transmitted_per_refresh),
        )),
        plain_line(terman_common::builtin_htop_overview_uptime_hint(
            &format_duration(snapshot.uptime),
        )),
        plain_line(terman_common::builtin_htop_overview_load_hint(
            snapshot.load_average.one,
            snapshot.load_average.five,
            snapshot.load_average.fifteen,
        )),
    ];
    lines.extend(core_meter_lines(
        snapshot.cpu_cores.as_slice(),
        core_rows,
        core_columns,
        content_width,
    ));
    lines.push(title_line(
        terman_common::builtin_htop_overview_top_processes_hint(),
    ));
    lines.push(process_header_line(sort, command_mode));
    for (offset, row) in snapshot
        .processes
        .iter()
        .skip(process_start)
        .take(process_rows)
        .enumerate()
    {
        let index = process_start + offset;
        lines.push(process_line(
            row,
            index == selected,
            snapshot.total_memory,
            content_width,
            tagged_pids.contains(row.pid.as_str()),
            command_mode,
        ));
    }
    render_block(
        frame,
        area,
        terman_common::builtin_htop_tab_overview_hint(),
        lines,
    );
}

fn render_block(
    frame: &mut Frame<'_>,
    area: Rect,
    title: String,
    lines: Vec<Line<'static>>,
) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn title_line(text: String) -> Line<'static> {
    Line::from(Span::styled(
        text,
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))
}

fn plain_line(text: String) -> Line<'static> {
    Line::from(Span::raw(text))
}