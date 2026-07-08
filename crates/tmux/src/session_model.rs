#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TmuxSessionEvent {
    Output(Vec<u8>),
    Resize { cols: u16, rows: u16 },
    Detach,
    Exit(i32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TmuxControlEvent {
    Input(Vec<u8>),
    Resize { cols: u16, rows: u16 },
    NewWindow { index: u32, name: String, command: Option<String> },
    RenameWindow { index: u32, name: String },
    KillWindow { index: u32 },
    SelectWindow { index: u32 },
    Terminate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TmuxSessionStatus {
    pub(crate) replay_bytes: usize,
    pub(crate) attached_clients: u32,
    pub(crate) windows: u32,
    pub(crate) active_window: u32,
    pub(crate) window_indexes: Vec<u32>,
    pub(crate) window_names: Vec<String>,
    pub(crate) cols: Option<u16>,
    pub(crate) rows: Option<u16>,
}
