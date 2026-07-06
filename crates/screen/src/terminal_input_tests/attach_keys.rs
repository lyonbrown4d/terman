use super::super::{ScreenInputAction, ScreenInputDecoder};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
#[test]
fn detects_screen_license_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let license = KeyEvent::new(KeyCode::Char(','), KeyModifiers::empty());

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(license), Some(ScreenInputAction::License));
}

#[test]
fn detects_screen_dumptermcap_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let dumptermcap = KeyEvent::new(KeyCode::Char('.'), KeyModifiers::empty());

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(
        decoder.decode_key(dumptermcap),
        Some(ScreenInputAction::DumpTermcap)
    );
}

#[test]
fn detects_screen_width_toggle_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let width = KeyEvent::new(KeyCode::Char('W'), KeyModifiers::SHIFT);

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(width), Some(ScreenInputAction::WidthToggle));
}

#[test]
fn detects_screen_fit_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let fit = KeyEvent::new(KeyCode::Char('F'), KeyModifiers::SHIFT);

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(fit), Some(ScreenInputAction::Fit));
}

#[test]
fn sends_xon_and_xoff_prefixes() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let xon = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
    let xoff = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(xon), Some(ScreenInputAction::Bytes(vec![0x11])));
    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(xoff), Some(ScreenInputAction::Bytes(vec![0x13])));
}

#[test]
fn detects_screen_log_toggle_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let log = KeyEvent::new(KeyCode::Char('H'), KeyModifiers::SHIFT);

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(log), Some(ScreenInputAction::LogToggle));
}
#[test]
fn detects_screen_ctrl_paste_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let paste = KeyEvent::new(KeyCode::Char(']'), KeyModifiers::CONTROL);

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(paste), Some(ScreenInputAction::Paste));
}
#[test]
fn detects_screen_redisplay_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let redisplay = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL);

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(redisplay), Some(ScreenInputAction::Redisplay));
}
#[test]
fn detects_screen_ctrl_windows_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let windows = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::CONTROL);

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(windows), Some(ScreenInputAction::Windows));
}
#[test]
fn detects_screen_ctrl_next_previous_prefixes() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let next = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL);
    let previous = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL);

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(next), Some(ScreenInputAction::NextWindow));
    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(
        decoder.decode_key(previous),
        Some(ScreenInputAction::PreviousWindow)
    );
}
#[test]
fn detects_screen_ctrl_window_lifecycle_prefixes() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let new_window = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    let detach = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
    let kill = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(new_window), Some(ScreenInputAction::NewWindow));
    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(detach), Some(ScreenInputAction::Detach));
    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(kill), Some(ScreenInputAction::Kill));
}
#[test]
fn detects_screen_ctrl_info_time_prefixes() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let info = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::CONTROL);
    let time = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL);

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(info), Some(ScreenInputAction::Info));

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(time), Some(ScreenInputAction::Time));
}
#[test]
fn detects_screen_last_message_prefixes() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let last_message = KeyEvent::new(KeyCode::Char('m'), KeyModifiers::empty());
    let ctrl_last_message = KeyEvent::new(KeyCode::Char('m'), KeyModifiers::CONTROL);

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(last_message), Some(ScreenInputAction::LastMessage));
    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(
        decoder.decode_key(ctrl_last_message),
        Some(ScreenInputAction::LastMessage)
    );
}