use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn terminal_text_width(text: &str) -> u16 {
    UnicodeWidthStr::width(text).min(u16::MAX as usize) as u16
}

pub fn fit_terminal_text(text: &str, width: usize) -> String {
    let mut output = String::new();
    let mut used = 0usize;
    for ch in text.chars() {
        let next = used.saturating_add(UnicodeWidthChar::width(ch).unwrap_or(0));
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
