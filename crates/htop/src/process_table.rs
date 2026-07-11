use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::{
    format::{format_bytes, format_duration},
    model::{ProcessRow, SortMode},
};

const PID_WIDTH: u16 = 8;
const PPID_WIDTH: u16 = 8;
const USER_WIDTH: u16 = 12;
const NICE_WIDTH: u16 = 4;
const STATE_WIDTH: u16 = 3;
const CPU_WIDTH: u16 = 6;
const MEM_WIDTH: u16 = 6;
const RES_WIDTH: u16 = 9;
const TIME_WIDTH: u16 = 10;
const COMMAND_START: u16 = PID_WIDTH + PPID_WIDTH + USER_WIDTH + NICE_WIDTH + STATE_WIDTH + CPU_WIDTH + MEM_WIDTH + RES_WIDTH + TIME_WIDTH;

pub(crate) fn sort_at_column(column: u16) -> Option<SortMode> {
    let pid_end = PID_WIDTH;
    let ppid_end = pid_end + PPID_WIDTH;
    let user_end = ppid_end + USER_WIDTH;
    let nice_end = user_end + NICE_WIDTH;
    let state_end = nice_end + STATE_WIDTH;
    let cpu_end = state_end + CPU_WIDTH;
    let mem_end = cpu_end + MEM_WIDTH;
    let res_end = mem_end + RES_WIDTH;
    let time_end = res_end + TIME_WIDTH;
    match column {
        c if c < pid_end => Some(SortMode::Pid),
        c if c < ppid_end => Some(SortMode::ParentPid),
        c if c < user_end => Some(SortMode::User),
        c if c < nice_end => Some(SortMode::Nice),
        c if c < state_end => Some(SortMode::State),
        c if c < cpu_end => Some(SortMode::Cpu),
        c if c < mem_end || c < res_end => Some(SortMode::Memory),
        c if c < time_end => Some(SortMode::Time),
        _ => Some(SortMode::Name),
    }
}

pub(crate) fn process_header_line(sort: SortMode) -> Line<'static> {
    Line::from(vec![
        header_span(format!("{:<8}", "PID"), sort == SortMode::Pid),
        header_span(format!("{:<8}", "PPID"), sort == SortMode::ParentPid),
        header_span(format!("{:<11} ", "USER"), sort == SortMode::User),
        header_span(format!("{:>3} ", "NI"), sort == SortMode::Nice),
        header_span(" S ".to_string(), sort == SortMode::State),
        header_span(format!("{:>5} ", "CPU%"), sort == SortMode::Cpu),
        header_span(format!("{:>5} ", "MEM%"), sort == SortMode::Memory),
        header_span(format!("{:>8} ", "RES"), sort == SortMode::Memory),
        header_span(format!("{:>9} ", "TIME+"), sort == SortMode::Time),
        header_span("COMMAND".to_string(), sort == SortMode::Name),
    ])
}

pub(crate) fn process_line(row: &ProcessRow, selected: bool, total_memory: u64, table_width: u16, tagged: bool) -> Line<'static> {
    let memory_percent = memory_percent(row.memory, total_memory);
    let state = status_char(row.status.as_str());
    let nice = row.nice.map(|value| value.to_string()).unwrap_or_else(|| "-".to_string());
    let ppid = row.parent_pid.as_deref().unwrap_or("-");
    let user = user_cell(row.user.as_str());
    let command = command_cell(tree_name(row, command_text(row)).as_str(), table_width);
    let text = process_text(row, ppid, user.as_str(), nice.as_str(), state.as_str(), memory_percent, command.as_str());
    if selected || tagged {
        return Line::from(Span::styled(text, row_style(tagged)));
    }
    Line::from(vec![
        Span::styled(format!("{:<8}", row.pid), Style::default().fg(Color::Gray)),
        Span::styled(format!("{:<8}", ppid), Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{} ", user), Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{:>3} ", nice), Style::default().fg(Color::Yellow)),
        Span::raw(" "),
        Span::styled(format!("{state:<1}"), status_style(state.as_str())),
        Span::raw(" "),
        Span::styled(format!("{:>5.1} ", row.cpu), usage_style(row.cpu as f64, 100.0)),
        Span::styled(format!("{:>5.1} ", memory_percent), usage_style(memory_percent, 100.0)),
        Span::styled(format!("{:>8} ", format_bytes(row.memory)), Style::default().fg(Color::White)),
        Span::styled(format!("{:>9} ", format_duration(row.run_time)), Style::default().fg(Color::White)),
        Span::raw(command),
    ])
}

fn process_text(row: &ProcessRow, ppid: &str, user: &str, nice: &str, state: &str, memory_percent: f64, command: &str) -> String {
    format!(
        "{:<8}{:<8}{} {:>3}  {:<1} {:>5.1} {:>5.1} {:>8} {:>9} {}",
        row.pid,
        ppid,
        user,
        nice,
        state,
        row.cpu,
        memory_percent,
        format_bytes(row.memory),
        format_duration(row.run_time),
        command
    )
}

fn user_cell(user: &str) -> String {
    let width = USER_WIDTH.saturating_sub(1) as usize;
    terman_common::fit_terminal_text(
        terman_common::truncate_terminal_text(user, width).as_str(),
        width,
    )
}
fn command_cell(command: &str, table_width: u16) -> String {
    let width = table_width.saturating_sub(COMMAND_START).max(1) as usize;
    terman_common::fit_terminal_text(terman_common::truncate_terminal_text(command, width).as_str(), width)
}

fn header_span(text: String, active: bool) -> Span<'static> {
    let style = if active {
        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    };
    Span::styled(text, style)
}

fn memory_percent(memory: u64, total_memory: u64) -> f64 {
    if total_memory == 0 { 0.0 } else { memory as f64 * 100.0 / total_memory as f64 }
}

fn command_text(row: &ProcessRow) -> &str {
    if row.command.is_empty() { row.name.as_str() } else { row.command.as_str() }
}

fn tree_name(row: &ProcessRow, name: &str) -> String {
    if row.depth == 0 && !row.has_children {
        return name.to_string();
    }
    let marker = match (row.has_children, row.collapsed) {
        (true, true) => "[+]",
        (true, false) => "[-]",
        (false, _) => "+-",
    };
    format!("{}{} {}", "  ".repeat(row.depth.min(12)), marker, name)
}

fn status_char(status: &str) -> String {
    status.chars().next().map(|char| char.to_ascii_uppercase().to_string()).unwrap_or_else(|| "-".to_string())
}

fn status_style(status: &str) -> Style {
    match status {
        "R" => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        "S" | "I" => Style::default().fg(Color::Cyan),
        "D" => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        "Z" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        "T" => Style::default().fg(Color::Magenta),
        _ => Style::default().fg(Color::DarkGray),
    }
}

fn usage_style(value: f64, max: f64) -> Style {
    if max <= 0.0 {
        return Style::default().fg(Color::DarkGray);
    }
    match value / max {
        ratio if ratio >= 0.85 => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ratio if ratio >= 0.60 => Style::default().fg(Color::Yellow),
        _ => Style::default().fg(Color::Green),
    }
}

fn row_style(tagged: bool) -> Style {
    let background = if tagged { Color::Yellow } else { Color::Green };
    Style::default().fg(Color::Black).bg(background)
}

#[cfg(test)]
mod tests {
    use super::command_cell;

    #[test]
    fn clips_wide_command_to_table_width() {
        let cell = command_cell("服务服务服务", 67);
        assert_eq!(terman_common::terminal_text_width(&cell), 5);
        assert!(cell.ends_with("..."));
    }
}
