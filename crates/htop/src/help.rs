use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub(crate) fn draw(frame: &mut Frame<'_>) {
    let lines = terman_common::builtin_htop_help_panel_hint()
        .lines()
        .map(help_line)
        .collect::<Vec<_>>();
    let block = Block::default()
        .title(terman_common::builtin_htop_footer_help_hint())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: false }), frame.area());
}

fn help_line(text: &str) -> Line<'static> {
    if text.starts_with("F") || text.starts_with("Tab") || text.starts_with("1-4") || text.starts_with("Esc") || text.starts_with("+") {
        Line::from(Span::styled(
            text.to_string(),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ))
    } else {
        Line::from(Span::raw(text.to_string()))
    }
}
