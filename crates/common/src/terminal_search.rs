use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

#[derive(Debug, Eq, PartialEq)]
pub enum TerminalSearchAction {
    Unhandled,
    Handled,
    MoveTo { line: usize, col: usize },
}

pub struct TerminalTextSearch {
    query: String,
    input: Option<String>,
    forward: bool,
}

impl Default for TerminalTextSearch {
    fn default() -> Self {
        Self {
            query: String::new(),
            input: None,
            forward: true,
        }
    }
}

impl TerminalTextSearch {
    pub fn handle_key(
        &mut self,
        key: &KeyEvent,
        lines: &[String],
        current: (usize, usize),
    ) -> TerminalSearchAction {
        if key.kind != KeyEventKind::Press {
            return TerminalSearchAction::Unhandled;
        }
        if self.input.is_some() {
            return self.handle_input(key, lines, current);
        }
        if key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            return TerminalSearchAction::Unhandled;
        }
        match key.code {
            KeyCode::Char('/') => {
                self.start(true);
                TerminalSearchAction::Handled
            }
            KeyCode::Char('?') => {
                self.start(false);
                TerminalSearchAction::Handled
            }
            KeyCode::Char('n') => self.repeat(lines, current, self.forward),
            KeyCode::Char('N') => self.repeat(lines, current, !self.forward),
            _ => TerminalSearchAction::Unhandled,
        }
    }

    pub fn status(&self) -> String {
        if let Some(input) = self.input.as_ref() {
            let prefix = if self.forward { '/' } else { '?' };
            return format!("{prefix}{input}_");
        }
        if self.query.is_empty() {
            "-".to_string()
        } else {
            self.query.clone()
        }
    }

    fn start(&mut self, forward: bool) {
        self.forward = forward;
        self.input = Some(String::new());
    }

    fn handle_input(
        &mut self,
        key: &KeyEvent,
        lines: &[String],
        current: (usize, usize),
    ) -> TerminalSearchAction {
        match key.code {
            KeyCode::Esc => {
                self.input = None;
                TerminalSearchAction::Handled
            }
            KeyCode::Backspace => {
                if let Some(input) = self.input.as_mut() {
                    input.pop();
                }
                TerminalSearchAction::Handled
            }
            KeyCode::Enter => {
                let query = self.input.take().unwrap_or_default();
                if query.is_empty() {
                    return TerminalSearchAction::Handled;
                }
                self.query = query;
                self.repeat(lines, current, self.forward)
            }
            KeyCode::Char(character)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                if let Some(input) = self.input.as_mut() {
                    input.push(character);
                }
                TerminalSearchAction::Handled
            }
            _ => TerminalSearchAction::Handled,
        }
    }

    fn repeat(
        &self,
        lines: &[String],
        current: (usize, usize),
        forward: bool,
    ) -> TerminalSearchAction {
        if self.query.is_empty() {
            return TerminalSearchAction::Handled;
        }
        match find_match(lines, &self.query, current, forward) {
            Some((line, col)) => TerminalSearchAction::MoveTo { line, col },
            None => TerminalSearchAction::Handled,
        }
    }
}

fn find_match(
    lines: &[String],
    query: &str,
    current: (usize, usize),
    forward: bool,
) -> Option<(usize, usize)> {
    let needle = query.to_lowercase();
    if needle.is_empty() {
        return None;
    }

    let mut matches = Vec::new();
    for (line_index, line) in lines.iter().enumerate() {
        let folded = line.to_lowercase();
        let mut offset = 0;
        while let Some(relative) = folded[offset..].find(&needle) {
            let byte_index = offset + relative;
            let col = folded[..byte_index].chars().count();
            matches.push((line_index, col));
            let step = folded[byte_index..]
                .chars()
                .next()
                .map(char::len_utf8)
                .unwrap_or(1);
            offset = byte_index + step;
        }
    }

    if forward {
        matches
            .iter()
            .copied()
            .find(|position| *position > current)
            .or_else(|| matches.first().copied())
    } else {
        matches
            .iter()
            .rev()
            .copied()
            .find(|position| *position < current)
            .or_else(|| matches.last().copied())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_search_wraps() {
        let lines = vec!["alpha".to_string(), "beta alpha".to_string()];
        assert_eq!(
            find_match(&lines, "alpha", (1, 5), true),
            Some((0, 0))
        );
    }

    #[test]
    fn backward_search_wraps() {
        let lines = vec!["alpha".to_string(), "beta alpha".to_string()];
        assert_eq!(
            find_match(&lines, "alpha", (0, 0), false),
            Some((1, 5))
        );
    }
}