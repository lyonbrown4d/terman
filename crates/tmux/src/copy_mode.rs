use std::io::{self, Write};

use crate::copy_history::{char_index_at_column, terminal_history};
use terman_common::{TerminalSearchAction, TerminalTextSearch};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind};
use unicode_width::UnicodeWidthChar;

pub(crate) enum TmuxCopyResult {
    Continue,
    Cancel,
    Copy(Vec<u8>),
}

pub(crate) struct TmuxCopyMode {
    lines: Vec<String>,
    top: usize,
    cursor_line: usize,
    cursor_col: usize,
    anchor: Option<(usize, usize)>,
    search: TerminalTextSearch,
    cols: u16,
    rows: u16,
}

impl TmuxCopyMode {
    pub(crate) fn from_replay(replay: &[u8], cols: u16, rows: u16) -> Self {
        let cols = cols.max(1);
        let rows = rows.max(2);
        let lines = terminal_history(replay, cols, rows);
        let cursor_line = lines.len().saturating_sub(1);
        let mut mode = Self {
            lines,
            top: 0,
            cursor_line,
            cursor_col: 0,
            anchor: None,
            search: TerminalTextSearch::default(),
            cols,
            rows,
        };
        mode.ensure_visible();
        mode
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> TmuxCopyResult {
        if key.kind != KeyEventKind::Press {
            return TmuxCopyResult::Continue;
        }

        match self.search.handle_key(
            &key,
            &self.lines,
            (self.cursor_line, self.cursor_col),
        ) {
            TerminalSearchAction::Unhandled => {}
            TerminalSearchAction::Handled => {
                return TmuxCopyResult::Continue;
            }
            TerminalSearchAction::MoveTo { line, col } => {
                self.cursor_line = line;
                self.cursor_col = col;
                self.clamp_cursor();
                self.ensure_visible();
                return TmuxCopyResult::Continue;
            }
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => return TmuxCopyResult::Cancel,
            KeyCode::Up | KeyCode::Char('k') => self.move_vertical(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_vertical(1),
            KeyCode::Left | KeyCode::Char('h') => self.move_left(),
            KeyCode::Right | KeyCode::Char('l') => self.move_right(),
            KeyCode::PageUp => self.move_vertical(-(self.data_rows() as isize)),
            KeyCode::PageDown => self.move_vertical(self.data_rows() as isize),
            KeyCode::Home | KeyCode::Char('0') => self.cursor_col = 0,
            KeyCode::End | KeyCode::Char('$') => self.cursor_col = self.current_line_len(),
            KeyCode::Char('g') => self.cursor_line = 0,
            KeyCode::Char('G') => self.cursor_line = self.lines.len().saturating_sub(1),
            KeyCode::Enter | KeyCode::Char(' ') => {
                if self.anchor.is_none() {
                    self.anchor = Some((self.cursor_line, self.cursor_col));
                } else {
                    return TmuxCopyResult::Copy(self.selected_text().into_bytes());
                }
            }
            _ => {}
        }
        self.clamp_cursor();
        self.ensure_visible();
        TmuxCopyResult::Continue
    }

    pub(crate) fn handle_mouse(&mut self, event: MouseEvent) -> bool {
        match event.kind {
            MouseEventKind::ScrollUp => self.move_vertical(-3),
            MouseEventKind::ScrollDown => self.move_vertical(3),
            MouseEventKind::Down(MouseButton::Left)
                if event.row < self.data_rows() && event.column < self.cols =>
            {
                self.cursor_line = (self.top + usize::from(event.row))
                    .min(self.lines.len().saturating_sub(1));
                self.cursor_col = char_index_at_column(
                    &self.lines[self.cursor_line],
                    usize::from(event.column),
                );
            }
            _ => return false,
        }
        self.clamp_cursor();
        self.ensure_visible();
        true
    }

    pub(crate) fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols.max(1);
        self.rows = rows.max(2);
        self.ensure_visible();
    }

    pub(crate) fn render(&self) -> io::Result<()> {
        let mut output = b"[?25l[0m[2J[H".to_vec();
        for row in 0..self.data_rows() {
            let line_index = self.top + usize::from(row);
            move_to(&mut output, row, 0);
            if let Some(line) = self.lines.get(line_index) {
                self.render_line(&mut output, line_index, line);
            }
            output.extend_from_slice(b"[0m");
        }
        self.render_status(&mut output);
        let mut stdout = io::stdout().lock();
        stdout.write_all(&output)?;
        stdout.flush()
    }

    fn render_line(&self, output: &mut Vec<u8>, line_index: usize, line: &str) {
        let mut used = 0usize;
        let mut styled = false;
        for (char_index, ch) in line.chars().enumerate() {
            let width = ch.width().unwrap_or(0);
            if used.saturating_add(width) > usize::from(self.cols) {
                break;
            }
            let selected = self.is_selected(line_index, char_index);
            let cursor = line_index == self.cursor_line && char_index == self.cursor_col;
            let next_styled = selected || cursor;
            if next_styled != styled {
                output.extend_from_slice(if next_styled { b"[7m" } else { b"[0m" });
                styled = next_styled;
            }
            let mut encoded = [0; 4];
            output.extend_from_slice(ch.encode_utf8(&mut encoded).as_bytes());
            used = used.saturating_add(width);
        }
        if line_index == self.cursor_line && self.cursor_col >= line.chars().count() {
            output.extend_from_slice(b"[7m [0m");
        }
    }

    fn render_status(&self, output: &mut Vec<u8>) {
        move_to(output, self.rows.saturating_sub(1), 0);
        let status = terman_common::builtin_tmux_copy_status_hint(
            self.cursor_line.saturating_add(1),
            self.lines.len(),
            self.anchor.is_some(),
            &self.search.status(),
        );
        let status = terman_common::fit_terminal_text(&status, usize::from(self.cols));
        output.extend_from_slice(b"[30;46m");
        output.extend_from_slice(status.as_bytes());
        output.extend_from_slice(b"[K[0m");
    }

    fn move_vertical(&mut self, delta: isize) {
        self.cursor_line = self
            .cursor_line
            .saturating_add_signed(delta)
            .min(self.lines.len().saturating_sub(1));
    }

    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.current_line_len();
        }
    }

    fn move_right(&mut self) {
        if self.cursor_col < self.current_line_len() {
            self.cursor_col += 1;
        } else if self.cursor_line + 1 < self.lines.len() {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    fn clamp_cursor(&mut self) {
        self.cursor_line = self.cursor_line.min(self.lines.len().saturating_sub(1));
        self.cursor_col = self.cursor_col.min(self.current_line_len());
    }

    fn ensure_visible(&mut self) {
        let visible = usize::from(self.data_rows()).max(1);
        if self.cursor_line < self.top {
            self.top = self.cursor_line;
        } else if self.cursor_line >= self.top.saturating_add(visible) {
            self.top = self.cursor_line.saturating_add(1).saturating_sub(visible);
        }
        self.top = self.top.min(self.lines.len().saturating_sub(visible));
    }

    fn data_rows(&self) -> u16 {
        self.rows.saturating_sub(1)
    }

    fn current_line_len(&self) -> usize {
        self.lines
            .get(self.cursor_line)
            .map(|line| line.chars().count())
            .unwrap_or(0)
    }

    fn is_selected(&self, line: usize, col: usize) -> bool {
        let Some(anchor) = self.anchor else {
            return false;
        };
        let cursor = (self.cursor_line, self.cursor_col);
        let (start, end) = if anchor <= cursor {
            (anchor, cursor)
        } else {
            (cursor, anchor)
        };
        (line, col) >= start && (line, col) <= end
    }

    fn selected_text(&self) -> String {
        let anchor = self.anchor.unwrap_or((self.cursor_line, self.cursor_col));
        let cursor = (self.cursor_line, self.cursor_col);
        let (start, end) = if anchor <= cursor {
            (anchor, cursor)
        } else {
            (cursor, anchor)
        };
        let mut selected = String::new();
        for line_index in start.0..=end.0 {
            let Some(line) = self.lines.get(line_index) else {
                continue;
            };
            let start_col = if line_index == start.0 { start.1 } else { 0 };
            let end_col = if line_index == end.0 {
                end.1.saturating_add(1)
            } else {
                line.chars().count()
            };
            selected.extend(line.chars().skip(start_col).take(end_col.saturating_sub(start_col)));
            if line_index < end.0 {
                selected.push('\n');
            }
        }
        selected
    }
}


fn move_to(output: &mut Vec<u8>, row: u16, col: u16) {
    let _ = write!(output, "[{};{}H", row.saturating_add(1), col.saturating_add(1));
}