use crossterm::event::KeyCode;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::model::SortMode;

const SORT_MODES: [SortMode; 6] = [
    SortMode::Cpu,
    SortMode::Memory,
    SortMode::Time,
    SortMode::Io,
    SortMode::Pid,
    SortMode::Name,
];

pub(crate) enum SortMenuAction {
    Continue,
    Apply(SortMode),
    Cancel,
}

pub(crate) fn handle_key(cursor: &mut SortMode, code: KeyCode) -> SortMenuAction {
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            *cursor = adjacent_sort(*cursor, false);
            SortMenuAction::Continue
        }
        KeyCode::Down | KeyCode::Char('j') => {
            *cursor = adjacent_sort(*cursor, true);
            SortMenuAction::Continue
        }
        KeyCode::Enter => SortMenuAction::Apply(*cursor),
        KeyCode::Esc | KeyCode::F(6) => SortMenuAction::Cancel,
        _ => SortMenuAction::Continue,
    }
}

pub(crate) fn draw(frame: &mut Frame<'_>, cursor: SortMode) {
    let area = centered_rect(frame.area(), 42, 10);
    let mut lines = vec![Line::from(Span::styled(
        terman_common::builtin_htop_sort_menu_help_hint(),
        Style::default().fg(Color::DarkGray),
    ))];

    for mode in SORT_MODES {
        lines.push(sort_line(mode, cursor == mode));
    }

    frame.render_widget(Clear, area);
    frame.render_widget(Paragraph::new(lines).block(sort_block()), area);
}


pub(crate) fn mode_at(area: Rect, column: u16, row: u16) -> Option<SortMode> {
    let menu = centered_rect(area, 42, 10);
    let inside_x = column > menu.x && column < menu.x.saturating_add(menu.width).saturating_sub(1);
    if !inside_x || row < menu.y.saturating_add(2) {
        return None;
    }
    let index = row.saturating_sub(menu.y + 2) as usize;
    SORT_MODES.get(index).copied()
}
fn sort_line(mode: SortMode, selected: bool) -> Line<'static> {
    let marker = if selected { ">" } else { " " };
    let text = format!(" {marker} {}", mode.label());
    if selected {
        Line::from(Span::styled(
            text,
            Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD),
        ))
    } else {
        Line::from(Span::raw(text))
    }
}

fn sort_block() -> Block<'static> {
    Block::default()
        .title(terman_common::builtin_htop_sort_menu_title_hint())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
}

fn adjacent_sort(current: SortMode, forward: bool) -> SortMode {
    let index = SORT_MODES.iter().position(|mode| *mode == current).unwrap_or(0);
    let next = if forward {
        (index + 1) % SORT_MODES.len()
    } else if index == 0 {
        SORT_MODES.len() - 1
    } else {
        index - 1
    };
    SORT_MODES[next]
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height.min(area.height)),
            Constraint::Min(0),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(width.min(area.width)),
            Constraint::Min(0),
        ])
        .split(vertical[1]);
    horizontal[1]
}

#[cfg(test)]
mod tests {
    use super::{SortMenuAction, handle_key};
    use crate::model::SortMode;
    use crossterm::event::KeyCode;

    #[test]
    fn navigates_sort_modes() {
        let mut cursor = SortMode::Cpu;
        assert!(matches!(handle_key(&mut cursor, KeyCode::Down), SortMenuAction::Continue));
        assert_eq!(cursor, SortMode::Memory);
        assert!(matches!(handle_key(&mut cursor, KeyCode::Up), SortMenuAction::Continue));
        assert_eq!(cursor, SortMode::Cpu);
    }
}