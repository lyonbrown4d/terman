use super::{ScreenInputAction, ScreenInputDecoder, key_to_bytes};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[test]
fn maps_control_char_to_control_byte() {
    let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);

    assert_eq!(key_to_bytes(key), Some(vec![0x03]));
}

#[test]
fn maps_arrow_key_to_escape_sequence() {
    let key = KeyEvent::new(KeyCode::Up, KeyModifiers::empty());

    assert_eq!(key_to_bytes(key), Some(vec![0x1b, b'[', b'A']));
}

#[test]
fn detects_screen_reset_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let reset = KeyEvent::new(KeyCode::Char('Z'), KeyModifiers::SHIFT);

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(reset), Some(ScreenInputAction::Reset));
}

#[test]
fn detects_screen_clear_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let clear = KeyEvent::new(KeyCode::Char('C'), KeyModifiers::SHIFT);

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(clear), Some(ScreenInputAction::Clear));
}

#[test]
fn detects_screen_detach_all_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let detach_all = KeyEvent::new(KeyCode::Char('D'), KeyModifiers::SHIFT);

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(
        decoder.decode_key(detach_all),
        Some(ScreenInputAction::DetachAll)
    );
}

#[test]
fn detects_screen_detach_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let detach = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::empty());

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(detach), Some(ScreenInputAction::Detach));
}

#[test]
fn detects_screen_kill_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let kill = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::empty());

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(kill), Some(ScreenInputAction::Kill));
}

#[test]
fn detects_screen_hardcopy_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let hardcopy = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::empty());

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(
        decoder.decode_key(hardcopy),
        Some(ScreenInputAction::Hardcopy)
    );
}

#[test]
fn detects_screen_help_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let help = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::empty());

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(help), Some(ScreenInputAction::Help));
}

#[test]
fn sends_literal_prefix_when_prefix_is_repeated() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(
        decoder.decode_key(prefix),
        Some(ScreenInputAction::Bytes(vec![0x01]))
    );
}