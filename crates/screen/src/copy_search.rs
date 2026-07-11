use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ScreenCopySearchAction {
    Unhandled,
    Handled,
    MoveTo { line: usize, col: usize },
}

pub(crate) struct ScreenCopySearch {
    query: String,
    input: Option<String>,
    forward: bool,
}

impl Default for ScreenCopySearch {
    fn default() -> Self {
        Self {
            query: String::new(),
            input: None,
            forward: true,
        }
    }
}

impl ScreenCopySearch {
    pub(crate) fn handle_key(
        &mut self,
        key: &KeyEvent,
        lines: &[String],
        current: (usize, usize),
    ) -> ScreenCopySearchAction {
        if key.kind != KeyEventKind::Press {
            return ScreenCopySearchAction::Unhandled;
        }
        if self.input.is_some() {
            return self.handle_input(key, lines, current);
        }
        if key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
        {
            return ScreenCopySearchAction::Unhandled;
        }
        match key.code {
            KeyCode::Char('/') => {
                self.start(true);
                ScreenCopySearchAction::Handled
            }
            KeyCode::Char('?') => {
                self.start(false);
                ScreenCopySearchAction::Handled
            }
            KeyCode::Char('n') => self.repeat(lines, current, self.forward),
            KeyCode::Char('N') => self.repeat(lines, current, !self.forward),
            _ => ScreenCopySearchAction::Unhandled,
        }
    }

    pub(crate) fn status(&self) -> String {
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
    ) -> ScreenCopySearchAction {
        match key.code {
            KeyCode::Enter => {
                let input = self.input.take().unwrap_or_default();
                let query = input.trim();
                if !query.is_empty() {
                    self.query = query.to_string();
                }
                self.repeat(lines, current, self.forward)
            }
            KeyCode::Esc => {
                self.input = None;
                ScreenCopySearchAction::Handled
            }
            KeyCode::Backspace => {
                if let Some(input) = self.input.as_mut() {
                    input.pop();
                }
                ScreenCopySearchAction::Handled
            }
            KeyCode::Char(ch)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                if let Some(input) = self.input.as_mut() {
                    input.push(ch);
                }
                ScreenCopySearchAction::Handled
            }
            _ => ScreenCopySearchAction::Handled,
        }
    }

    fn repeat(
        &self,
        lines: &[String],
        current: (usize, usize),
        forward: bool,
    ) -> ScreenCopySearchAction {
        find_match(lines, self.query.as_str(), current, forward)
            .map(|(line, col)| ScreenCopySearchAction::MoveTo { line, col })
            .unwrap_or(ScreenCopySearchAction::Handled)
    }
}

fn find_match(
    lines: &[String],
    query: &str,
    current: (usize, usize),
    forward: bool,
) -> Option<(usize, usize)> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return None;
    }
    let mut first = None;
    let mut last = None;
    let mut after = None;
    let mut before = None;
    for (line_index, line) in lines.iter().enumerate() {
        for (col, (byte, _)) in line.char_indices().enumerate() {
            if !line[byte..].to_lowercase().starts_with(needle.as_str()) {
                continue;
            }
            let position = (line_index, col);
            first.get_or_insert(position);
            last = Some(position);
            if position > current && after.is_none() {
                after = Some(position);
            }
            if position < current {
                before = Some(position);
            }
        }
    }
    if forward {
        after.or(first)
    } else {
        before.or(last)
    }
}

#[cfg(test)]
mod tests {
    use super::find_match;

    #[test]
    fn finds_forward_matches_and_wraps() {
        let lines = vec!["alpha".to_string(), "Beta alpha".to_string()];
        assert_eq!(find_match(&lines, "ALPHA", (0, 0), true), Some((1, 5)));
        assert_eq!(find_match(&lines, "alpha", (1, 5), true), Some((0, 0)));
    }

    #[test]
    fn finds_backward_matches_and_wraps() {
        let lines = vec!["alpha".to_string(), "Beta alpha".to_string()];
        assert_eq!(find_match(&lines, "alpha", (1, 5), false), Some((0, 0)));
        assert_eq!(find_match(&lines, "alpha", (0, 0), false), Some((1, 5)));
    }
}