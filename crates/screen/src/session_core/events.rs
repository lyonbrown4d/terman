use std::path::PathBuf;

use crate::region_types::{ScreenRegionAxis, ScreenRegionFocus, ScreenRegionResize};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ScreenSessionEvent {
    Output(Vec<u8>),
    Resize { cols: u16, rows: u16 },
    Detach,
    Exit(i32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ScreenControlEvent {
    Input(Vec<u8>),
    BlankRegion,
    NewWindow { command: Option<String> },
    SetDefaultCwd { path: PathBuf },
    SetEnv { name: String, value: String },
    UnsetEnv { name: String },
    SetDefaultScrollback { lines: usize },
    SelectWindow { index: usize },
    NumberWindow { source: usize, index: usize },
    NextWindow,
    PreviousWindow,
    LastWindow,
    KillWindow,
    SplitRegion { axis: ScreenRegionAxis },
    FocusRegion { target: ScreenRegionFocus },
    RemoveRegion,
    OnlyRegion,
    ResizeRegion { resize: ScreenRegionResize },
    Resize { cols: u16, rows: u16 },
    Terminate,
}