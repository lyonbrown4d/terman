use crossterm::event::{
    KeyCode, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

#[derive(Default)]
pub(crate) struct EnvironmentViewState {
    content: Option<EnvironmentContent>,
    visible_rows: usize,
}

struct EnvironmentContent {
    pid: String,
    entries: Vec<String>,
    scroll: usize,
}

impl EnvironmentViewState {
    pub(crate) fn open(
        &mut self,
        pid: String,
        mut entries: Vec<String>,
    ) {
        entries.sort_by_key(|entry| entry.to_lowercase());
        self.content = Some(EnvironmentContent {
            pid,
            entries,
            scroll: 0,
        });
    }

    pub(crate) fn is_open(&self) -> bool {
        self.content.is_some()
    }

    pub(crate) fn handle_key(&mut self, code: KeyCode) -> bool {
        if self.content.is_none() {
            return false;
        }
        match code {
            KeyCode::Esc
            | KeyCode::F(10)
            | KeyCode::Char('e' | 'q') => self.content = None,
            KeyCode::Up | KeyCode::Char('k') => self.scroll_up(1),
            KeyCode::Down | KeyCode::Char('j') => self.scroll_down(1),
            KeyCode::PageUp => {
                self.scroll_up(self.visible_rows.max(1));
            }
            KeyCode::PageDown => {
                self.scroll_down(self.visible_rows.max(1));
            }
            KeyCode::Home => self.set_scroll(0),
            KeyCode::End => self.set_scroll(self.max_scroll()),
            _ => {}
        }
        true
    }

    pub(crate) fn handle_mouse(&mut self, event: MouseEvent) -> bool {
        if self.content.is_none() {
            return false;
        }
        match event.kind {
            MouseEventKind::ScrollUp => self.scroll_up(1),
            MouseEventKind::ScrollDown => self.scroll_down(1),
            MouseEventKind::Down(MouseButton::Middle | MouseButton::Right) => {
                self.content = None;
            }
            _ => {}
        }
        true
    }

    pub(crate) fn draw(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        let Some(content) = self.content.as_mut() else {
            return;
        };
        let title = format!(
            " {} | {} ",
            terman_common::builtin_htop_environment_title_hint(&content.pid),
            terman_common::builtin_htop_environment_help_hint(),
        );
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        self.visible_rows = usize::from(inner.height).max(1);
        content.scroll = content
            .scroll
            .min(max_scroll(content.entries.len(), self.visible_rows));

        let lines = if content.entries.is_empty() {
            vec![Line::from(Span::styled(
                terman_common::builtin_htop_environment_empty_hint(),
                Style::default().fg(Color::DarkGray),
            ))]
        } else {
            content
                .entries
                .iter()
                .skip(content.scroll)
                .take(self.visible_rows)
                .map(|entry| environment_line(entry, inner.width))
                .collect()
        };
        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
        frame.render_widget(Paragraph::new(lines), inner);
    }

    fn scroll_up(&mut self, amount: usize) {
        if let Some(content) = self.content.as_mut() {
            content.scroll = content.scroll.saturating_sub(amount);
        }
    }

    fn scroll_down(&mut self, amount: usize) {
        let maximum = self.max_scroll();
        if let Some(content) = self.content.as_mut() {
            content.scroll =
                content.scroll.saturating_add(amount).min(maximum);
        }
    }

    fn set_scroll(&mut self, scroll: usize) {
        let maximum = self.max_scroll();
        if let Some(content) = self.content.as_mut() {
            content.scroll = scroll.min(maximum);
        }
    }

    fn max_scroll(&self) -> usize {
        self.content
            .as_ref()
            .map(|content| {
                max_scroll(content.entries.len(), self.visible_rows)
            })
            .unwrap_or_default()
    }
}

fn max_scroll(entry_count: usize, visible_rows: usize) -> usize {
    entry_count.saturating_sub(visible_rows.max(1))
}

fn environment_line(entry: &str, width: u16) -> Line<'static> {
    let fitted = terman_common::fit_terminal_text(
        entry,
        usize::from(width),
    );
    let Some((name, value)) = fitted.split_once('=') else {
        return Line::from(Span::raw(fitted));
    };
    Line::from(vec![
        Span::styled(
            name.to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("=", Style::default().fg(Color::DarkGray)),
        Span::raw(value.to_string()),
    ])
}