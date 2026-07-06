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
fn detects_screen_resize_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let resize = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::empty());

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(resize), Some(ScreenInputAction::Resize));
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
fn detects_screen_new_window_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let new_window = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::empty());

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(new_window), Some(ScreenInputAction::NewWindow));
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
fn detects_screen_numeric_window_select_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let select = KeyEvent::new(KeyCode::Char('3'), KeyModifiers::empty());

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(select), Some(ScreenInputAction::SelectWindow(3)));
}
#[test]
fn detects_screen_next_window_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let next = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::empty());

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(next), Some(ScreenInputAction::NextWindow));
}

#[test]
fn detects_screen_previous_window_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let previous = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::empty());

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(
        decoder.decode_key(previous),
        Some(ScreenInputAction::PreviousWindow)
    );
}
#[test]
fn detects_screen_time_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let time = KeyEvent::new(KeyCode::Char('t'), KeyModifiers::empty());

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(time), Some(ScreenInputAction::Time));
}

#[test]
fn detects_screen_version_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let version = KeyEvent::new(KeyCode::Char('v'), KeyModifiers::empty());

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(version), Some(ScreenInputAction::Version));
}

#[test]
fn detects_screen_displays_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let displays = KeyEvent::new(KeyCode::Char('*'), KeyModifiers::empty());

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(displays), Some(ScreenInputAction::Displays));
}

#[test]
fn detects_screen_windows_prefix() {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let windows = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::empty());

    assert_eq!(decoder.decode_key(prefix), None);
    assert_eq!(decoder.decode_key(windows), Some(ScreenInputAction::Windows));
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