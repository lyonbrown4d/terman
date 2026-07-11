use crate::{
    model::{IoRow, ProcessRow, SocketRow, SortMode},
    render::Tab,
    signal_menu::SignalMenuState,
};

pub(crate) struct MouseContext<'a> {
    pub(crate) tab: &'a mut Tab,
    pub(crate) sort: &'a mut SortMode,
    pub(crate) sort_inverted: &'a mut bool,
    pub(crate) sort_menu_open: &'a mut bool,
    pub(crate) sort_cursor: &'a mut SortMode,
    pub(crate) sort_header_pressed: &'a mut Option<SortMode>,
    pub(crate) tree: &'a mut bool,
    pub(crate) help_open: &'a mut bool,
    pub(crate) selected: &'a mut usize,
    pub(crate) detail_scroll: &'a mut usize,
    pub(crate) io_scroll: &'a mut usize,
    pub(crate) network_scroll: &'a mut usize,
    pub(crate) processes: &'a [ProcessRow],
    pub(crate) io: &'a [IoRow],
    pub(crate) sockets: &'a [SocketRow],
    pub(crate) cpu_core_count: usize,
    pub(crate) filter: &'a str,
    pub(crate) search: &'a str,
    pub(crate) signal_menu: &'a mut Option<SignalMenuState>,
    pub(crate) refresh_ms: u64,
}