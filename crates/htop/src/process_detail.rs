use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::{
    format::{format_bytes, format_duration},
    model::ProcessRow,
};

pub(crate) fn process_detail_lines(
    row: Option<&ProcessRow>,
) -> Vec<Line<'static>> {
    let Some(row) = row else {
        return vec![muted_line(
            terman_common::builtin_htop_detail_none_hint(),
        )];
    };
    let read = terman_common::builtin_htop_detail_io_hint(
        &format_bytes(row.read_rate),
        &format_bytes(row.read_total),
    );
    let write = terman_common::builtin_htop_detail_io_hint(
        &format_bytes(row.written_rate),
        &format_bytes(row.written_total),
    );
    let mut lines = vec![
        detail_line("PID", row.pid.as_str()),
        detail_line("PPID", row.parent_pid.as_deref().unwrap_or("-")),
        detail_line(
            &terman_common::builtin_htop_detail_user_hint(),
            row.user.as_str(),
        ),
        detail_line(
            &terman_common::builtin_htop_detail_status_hint(),
            row.status.as_str(),
        ),
        detail_line("CPU", format!("{:.1}%", row.cpu).as_str()),
        detail_line(
            &terman_common::builtin_htop_detail_memory_hint(),
            format_bytes(row.memory).as_str(),
        ),
        detail_line(
            &terman_common::builtin_htop_detail_runtime_hint(),
            format_duration(row.run_time).as_str(),
        ),
        detail_line(
            &terman_common::builtin_htop_detail_read_hint(),
            read.as_str(),
        ),
        detail_line(
            &terman_common::builtin_htop_detail_write_hint(),
            write.as_str(),
        ),
    ];
    lines.extend(command_lines(row.command.as_str()));
    lines
}

fn command_lines(command: &str) -> Vec<Line<'static>> {
    const WIDTH: u16 = 96;
    if command.is_empty() {
        return vec![detail_line(
            &terman_common::builtin_htop_detail_command_hint(),
            "-",
        )];
    }
    let mut lines = Vec::new();
    let mut chunk = String::new();
    let mut width = 0u16;
    for ch in command.chars() {
        let ch_width = char_width(ch);
        if !chunk.is_empty() && width.saturating_add(ch_width) > WIDTH {
            lines.push(detail_line(
                &command_label(lines.is_empty()),
                chunk.as_str(),
            ));
            chunk.clear();
            width = 0;
        }
        chunk.push(ch);
        width = width.saturating_add(ch_width);
    }
    if !chunk.is_empty() {
        lines.push(detail_line(
            &command_label(lines.is_empty()),
            chunk.as_str(),
        ));
    }
    lines
}

fn char_width(ch: char) -> u16 {
    let mut buffer = [0u8; 4];
    terman_common::terminal_text_width(ch.encode_utf8(&mut buffer))
}

fn command_label(first: bool) -> String {
    if first {
        terman_common::builtin_htop_detail_command_hint()
    } else {
        String::new()
    }
}

fn detail_line(label: &str, value: &str) -> Line<'static> {
    let label_width = terman_common::terminal_text_width(label);
    let padding = usize::from(8u16.saturating_sub(label_width));
    Line::from(vec![
        Span::styled(
            format!("{label}{}", " ".repeat(padding)),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(value.to_string()),
    ])
}

fn muted_line(text: String) -> Line<'static> {
    Line::from(Span::styled(
        text,
        Style::default().fg(Color::DarkGray),
    ))
}

#[cfg(test)]
mod tests {
    use super::command_lines;

    #[test]
    fn wraps_wide_command_by_terminal_width() {
        let command = "服务".repeat(49);
        assert_eq!(command_lines(&command).len(), 2);
    }
}