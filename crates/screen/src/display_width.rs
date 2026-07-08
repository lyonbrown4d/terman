use unicode_width::UnicodeWidthStr;

pub(crate) fn text_width(text: &str) -> u16 {
    UnicodeWidthStr::width(text).min(u16::MAX as usize) as u16
}
