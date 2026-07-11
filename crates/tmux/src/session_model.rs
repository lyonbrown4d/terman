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
    SplitPane { window: u32, horizontal: bool, command: Option<String> },
    SelectPane { window: u32, pane: u32 },
    SwapPane { window: u32, source: u32, target: u32 },
    KillPane { window: u32, pane: u32 },
    TogglePaneZoom { window: u32, pane: u32 },
    ResizePane {
        window: u32,
        pane: u32,
        cols: Option<u16>,
        rows: Option<u16>,
    },
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TmuxPaneStatus {
    pub(crate) window_index: u32,
    pub(crate) window_name: String,
    pub(crate) active_pane: u32,
    pub(crate) pane_indexes: Vec<u32>,
}
