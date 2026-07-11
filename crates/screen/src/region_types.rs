use serde::{Deserialize, Serialize};

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