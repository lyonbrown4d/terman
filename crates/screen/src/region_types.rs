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
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) enum ScreenRegionResizeMode {
    Width,
    Height,
    Both,
    Local,
    Perpendicular,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) enum ScreenRegionResizeAmount {
    Delta(i32),
    Absolute(u16),
    DeltaPercent(i32),
    AbsolutePercent(u16),
    Equalize,
    Maximum,
    Minimum,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct ScreenRegionResize {
    pub(crate) mode: ScreenRegionResizeMode,
    pub(crate) amount: ScreenRegionResizeAmount,
}
