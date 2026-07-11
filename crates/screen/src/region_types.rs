use serde::{Deserialize, Serialize};

pub(crate) const BLANK_SCREEN_WINDOW_INDEX: usize = usize::MAX;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) enum ScreenRegionAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) enum ScreenRegionFocus {
    Next,
    Previous,
    First,
    Last,
}