use crossterm::event::{KeyCode, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

#[derive(Default)]
pub(crate) struct UserFilterState {
    selected: Option<String>,
    menu: Option<UserMenu>,
}

struct UserMenu {
    users: Vec<String>,
    cursor: usize,
    offset: usize,
}

enum MenuAction {
    Continue,
    Apply(Option<String>),
    Cancel,
}

impl UserFilterState {
    pub(crate) fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    pub(crate) fn open(&mut self, users: &[String]) {
        let users = users.to_vec();
        let cursor = self
            .selected
            .as_ref()
            .and_then(|selected| users.iter().position(|user| user == selected))
            .map(|index| index + 1)
            .unwrap_or(0);
        self.menu = Some(UserMenu { users, cursor, offset: 0 });
    }

    pub(crate) fn handle_key(&mut self, code: KeyCode) -> bool {
        let Some(menu) = self.menu.as_mut() else {
            return false;
        };
        let action = match code {
            KeyCode::Up | KeyCode::Char('k') => {
                menu.move_cursor(false);
                MenuAction::Continue
            }
            KeyCode::Down | KeyCode::Char('j') => {
                menu.move_cursor(true);
                MenuAction::Continue
            }
            KeyCode::Home => {
                menu.cursor = 0;
                MenuAction::Continue
            }
            KeyCode::End => {
                menu.cursor = menu.option_count().saturating_sub(1);
                MenuAction::Continue
            }
            KeyCode::Enter => MenuAction::Apply(menu.choice(menu.cursor)),
            KeyCode::Esc | KeyCode::Char('u') => MenuAction::Cancel,
            _ => MenuAction::Continue,
        };
        self.apply(action);
        true
    }

    pub(crate) fn handle_mouse(&mut self, event: &MouseEvent, terminal: Rect) -> bool {
        let Some(menu) = self.menu.as_mut() else {
            return false;
        };
        let area = menu_area(terminal, menu.option_count());
        let action = match event.kind {
            MouseEventKind::ScrollUp => {
                menu.move_cursor(false);
                menu.ensure_visible(area);
                MenuAction::Continue
            }
            MouseEventKind::ScrollDown => {
                menu.move_cursor(true);
                menu.ensure_visible(area);
                MenuAction::Continue
            }
            MouseEventKind::Down(MouseButton::Left) => menu
                .option_at(area, event.column, event.row)
                .map(|index| MenuAction::Apply(menu.choice(index)))
                .unwrap_or(MenuAction::Cancel),
            MouseEventKind::Down(MouseButton::Right) => MenuAction::Cancel,
            _ => MenuAction::Continue,
        };
        self.apply(action);
        true
    }

    pub(crate) fn draw(&mut self, frame: &mut Frame<'_>) {
        let Some(menu) = self.menu.as_mut() else {
            return;
        };
        let area = menu_area(frame.area(), menu.option_count());
        menu.ensure_visible(area);
        let mut lines = vec![Line::from(Span::styled(
            terman_common::builtin_htop_user_menu_help_hint(),
            Style::default().fg(Color::DarkGray),
        ))];
        let visible = visible_options(area);
        for index in menu.offset..menu.option_count().min(menu.offset + visible) {
            lines.push(user_line(menu.label(index), index == menu.cursor));
        }
        let block = Block::default()
            .title(terman_common::builtin_htop_user_menu_title_hint())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        frame.render_widget(Clear, area);
        frame.render_widget(Paragraph::new(lines).block(block), area);
    }

    fn apply(&mut self, action: MenuAction) {
        match action {
            MenuAction::Continue => {}
            MenuAction::Apply(selected) => {
                self.selected = selected;
                self.menu = None;
            }
            MenuAction::Cancel => self.menu = None,
        }
    }
}

impl UserMenu {
    fn option_count(&self) -> usize {
        self.users.len() + 1
    }

    fn choice(&self, index: usize) -> Option<String> {
        index.checked_sub(1).and_then(|index| self.users.get(index).cloned())
    }

    fn label(&self, index: usize) -> String {
        self.choice(index)
            .unwrap_or_else(terman_common::builtin_htop_all_users_hint)
    }

    fn move_cursor(&mut self, forward: bool) {
        let count = self.option_count();
        self.cursor = if forward {
            (self.cursor + 1) % count
        } else if self.cursor == 0 {
            count - 1
        } else {
            self.cursor - 1
        };
    }

    fn ensure_visible(&mut self, area: Rect) {
        let visible = visible_options(area).max(1);
        if self.cursor < self.offset {
            self.offset = self.cursor;
        } else if self.cursor >= self.offset + visible {
            self.offset = self.cursor + 1 - visible;
        }
    }

    fn option_at(&self, area: Rect, column: u16, row: u16) -> Option<usize> {
        let inside_x = column > area.x
            && column < area.x.saturating_add(area.width).saturating_sub(1);
        let start = area.y.saturating_add(2);
        let end = area.y.saturating_add(area.height).saturating_sub(1);
        if !inside_x || row < start || row >= end {
            return None;
        }
        let index = self.offset + usize::from(row - start);
        (index < self.option_count()).then_some(index)
    }
}

fn user_line(label: String, selected: bool) -> Line<'static> {
    let marker = if selected { ">" } else { " " };
    let text = format!(" {marker} {label}");
    let style = if selected {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    Line::from(Span::styled(text, style))
}

fn menu_area(area: Rect, options: usize) -> Rect {
    let height = u16::try_from(options)
        .unwrap_or(u16::MAX)
        .saturating_add(3)
        .min(area.height);
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(48.min(area.width)),
            Constraint::Min(0),
        ])
        .split(vertical[1]);
    horizontal[1]
}

fn visible_options(area: Rect) -> usize {
    usize::from(area.height.saturating_sub(3))
}
