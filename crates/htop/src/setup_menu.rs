use crossterm::{
    event::{KeyCode, MouseButton, MouseEvent, MouseEventKind},
    terminal,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

const REFRESH_STEPS: [u64; 7] = [100, 250, 500, 1_000, 2_000, 5_000, 10_000];
const ITEMS: [SetupItem; 3] = [
    SetupItem::Refresh,
    SetupItem::Tree,
    SetupItem::SortDirection,
];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum SetupItem {
    #[default]
    Refresh,
    Tree,
    SortDirection,
}

#[derive(Debug, Default)]
pub(crate) struct SetupMenuState {
    open: bool,
    cursor: SetupItem,
}

impl SetupMenuState {
    pub(crate) fn open(&mut self) {
        self.open = true;
    }

    pub(crate) fn handle_key(
        &mut self,
        code: KeyCode,
        refresh_ms: &mut u64,
        tree: &mut bool,
        sort_inverted: &mut bool,
    ) -> bool {
        if !self.open {
            if code == KeyCode::F(2) {
                self.open();
                return true;
            }
            return false;
        }
        match code {
            KeyCode::Esc | KeyCode::F(2) => self.open = false,
            KeyCode::Up | KeyCode::Char('k') => self.move_cursor(false),
            KeyCode::Down | KeyCode::Char('j') => self.move_cursor(true),
            KeyCode::Left | KeyCode::Char('-') => {
                self.apply(false, refresh_ms, tree, sort_inverted)
            }
            KeyCode::Right | KeyCode::Enter | KeyCode::Char('+' | '=' | ' ') => {
                self.apply(true, refresh_ms, tree, sort_inverted)
            }
            _ => {}
        }
        true
    }

    pub(crate) fn handle_mouse(
        &mut self,
        event: MouseEvent,
        refresh_ms: &mut u64,
        tree: &mut bool,
        sort_inverted: &mut bool,
    ) -> bool {
        if !self.open {
            return false;
        }
        match event.kind {
            MouseEventKind::ScrollUp => self.move_cursor(false),
            MouseEventKind::ScrollDown => self.move_cursor(true),
            MouseEventKind::Down(MouseButton::Left) => {
                self.click(event.column, event.row, true, refresh_ms, tree, sort_inverted)
            }
            MouseEventKind::Down(MouseButton::Right) => {
                self.click(event.column, event.row, false, refresh_ms, tree, sort_inverted)
            }
            MouseEventKind::Down(MouseButton::Middle) => self.open = false,
            _ => {}
        }
        true
    }

    pub(crate) fn draw(
        &self,
        frame: &mut Frame<'_>,
        refresh_ms: u64,
        tree: bool,
        sort_inverted: bool,
    ) {
        if !self.open {
            return;
        }
        let area = setup_area(frame.area());
        let lines = vec![
            Line::from(Span::styled(
                terman_common::builtin_htop_setup_help_hint(),
                Style::default().fg(Color::DarkGray),
            )),
            setting_line(
                terman_common::builtin_htop_setup_refresh_hint(),
                format!("{refresh_ms} ms"),
                self.cursor == SetupItem::Refresh,
            ),
            setting_line(
                terman_common::builtin_htop_setup_tree_hint(),
                terman_common::builtin_htop_setup_toggle_hint(tree),
                self.cursor == SetupItem::Tree,
            ),
            setting_line(
                terman_common::builtin_htop_setup_sort_direction_hint(),
                terman_common::builtin_htop_setup_direction_hint(sort_inverted),
                self.cursor == SetupItem::SortDirection,
            ),
        ];
        let block = Block::default()
            .title(terman_common::builtin_htop_setup_title_hint())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        frame.render_widget(Clear, area);
        frame.render_widget(Paragraph::new(lines).block(block), area);
    }

    fn click(
        &mut self,
        column: u16,
        row: u16,
        forward: bool,
        refresh_ms: &mut u64,
        tree: &mut bool,
        sort_inverted: &mut bool,
    ) {
        let (width, height) = terminal::size().unwrap_or((80, 24));
        let Some(item) = item_at(Rect::new(0, 0, width, height), column, row) else {
            self.open = false;
            return;
        };
        self.cursor = item;
        self.apply(forward, refresh_ms, tree, sort_inverted);
    }

    fn move_cursor(&mut self, forward: bool) {
        let index = ITEMS.iter().position(|item| *item == self.cursor).unwrap_or(0);
        self.cursor = if forward {
            ITEMS[(index + 1) % ITEMS.len()]
        } else {
            ITEMS[(index + ITEMS.len() - 1) % ITEMS.len()]
        };
    }

    fn apply(
        &self,
        forward: bool,
        refresh_ms: &mut u64,
        tree: &mut bool,
        sort_inverted: &mut bool,
    ) {
        match self.cursor {
            SetupItem::Refresh => *refresh_ms = adjacent_refresh(*refresh_ms, forward),
            SetupItem::Tree => *tree = !*tree,
            SetupItem::SortDirection => *sort_inverted = !*sort_inverted,
        }
    }
}

fn adjacent_refresh(current: u64, forward: bool) -> u64 {
    if forward {
        REFRESH_STEPS.iter().copied().find(|step| *step > current)
            .unwrap_or(REFRESH_STEPS[REFRESH_STEPS.len() - 1])
    } else {
        REFRESH_STEPS.iter().rev().copied().find(|step| *step < current)
            .unwrap_or(REFRESH_STEPS[0])
    }
}

fn setting_line(label: String, value: String, selected: bool) -> Line<'static> {
    let marker = if selected { ">" } else { " " };
    let style = if selected {
        Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    Line::from(Span::styled(format!(" {marker} {label}: {value}"), style))
}

fn item_at(area: Rect, column: u16, row: u16) -> Option<SetupItem> {
    let menu = setup_area(area);
    let inside_x = column > menu.x && column < menu.x.saturating_add(menu.width).saturating_sub(1);
    if !inside_x || row < menu.y.saturating_add(2) {
        return None;
    }
    ITEMS.get(row.saturating_sub(menu.y + 2) as usize).copied()
}

fn setup_area(area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(7.min(area.height)), Constraint::Min(0)])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(66.min(area.width)), Constraint::Min(0)])
        .split(vertical[1]);
    horizontal[1]
}