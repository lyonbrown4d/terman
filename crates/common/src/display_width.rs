use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn terminal_text_width(text: &str) -> u16 {
    UnicodeWidthStr::width(text).min(u16::MAX as usize) as u16
}

pub fn fit_terminal_text(text: &str, width: usize) -> String {
    let mut output = String::new();
    let mut used = 0usize;
    for ch in text.chars() {
        let next = used.saturating_add(terminal_char_width(ch));
        if next > width {
            break;
        }
        output.push(ch);
        used = next;
    }
    if used < width {
        output.push_str(&" ".repeat(width - used));
    }
    output
}

pub fn truncate_terminal_text(text: &str, width: usize) -> String {
    if terminal_text_width(text) as usize <= width {
        return text.to_string();
    }
    let marker = "...";
    let body_width = width.saturating_sub(terminal_text_width(marker) as usize);
    let mut output = String::new();
    let mut used = 0usize;
    for ch in text.chars() {
        let next = used.saturating_add(terminal_char_width(ch));
        if next > body_width {
            break;
        }
        output.push(ch);
        used = next;
    }
    output.push_str(marker);
    output
}

fn terminal_char_width(ch: char) -> usize {
    UnicodeWidthChar::width(ch).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{fit_terminal_text, terminal_text_width, truncate_terminal_text};

    #[test]
    fn truncates_wide_text_with_ellipsis() {
        assert_eq!(truncate_terminal_text("服务服务服务服务", 7), "服务...");
    }

    #[test]
    fn fits_wide_text_to_exact_width() {
        let text = fit_terminal_text("服务", 6);
        assert_eq!(terminal_text_width(&text), 6);
    }
}
