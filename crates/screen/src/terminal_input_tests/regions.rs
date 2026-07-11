use super::super::{ScreenInputAction, ScreenInputDecoder};
use crate::region_types::{ScreenRegionAxis, ScreenRegionFocus};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn decode(key: KeyEvent) -> Option<ScreenInputAction> {
    let mut decoder = ScreenInputDecoder::new();
    let prefix = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
    assert_eq!(decoder.decode_key(prefix), None);
    decoder.decode_key(key)
}

#[test]
fn maps_region_split_shortcuts() {
    assert_eq!(
        decode(KeyEvent::new(KeyCode::Char('S'), KeyModifiers::SHIFT)),
        Some(ScreenInputAction::SplitRegion(ScreenRegionAxis::Horizontal))
    );
    assert_eq!(
        decode(KeyEvent::new(KeyCode::Char('|'), KeyModifiers::SHIFT)),
        Some(ScreenInputAction::SplitRegion(ScreenRegionAxis::Vertical))
    );
}

#[test]
fn maps_region_management_shortcuts() {
    assert_eq!(
        decode(KeyEvent::new(KeyCode::Tab, KeyModifiers::empty())),
        Some(ScreenInputAction::FocusRegion(ScreenRegionFocus::Next))
    );
    assert_eq!(
        decode(KeyEvent::new(KeyCode::Char('X'), KeyModifiers::SHIFT)),
        Some(ScreenInputAction::RemoveRegion)
    );
    assert_eq!(
        decode(KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::SHIFT)),
        Some(ScreenInputAction::OnlyRegion)
    );
}