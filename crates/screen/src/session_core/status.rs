use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScreenWindowStatus {
    pub(crate) index: usize,
    pub(crate) title: Option<String>,
    pub(crate) active: bool,
    pub(crate) replay_bytes: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScreenSessionStatus {
    pub(crate) replay_bytes: usize,
    pub(crate) attach_clients: usize,
    pub(crate) cols: Option<u16>,
    pub(crate) rows: Option<u16>,
    pub(crate) scrollback_lines: usize,
    pub(crate) hardcopy_dir: Option<PathBuf>,
    pub(crate) hardcopy_append: bool,
    pub(crate) buffer_file: PathBuf,
    pub(crate) window_title: Option<String>,
    pub(crate) active_window: usize,
    pub(crate) windows: Vec<ScreenWindowStatus>,
}