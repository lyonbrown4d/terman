use std::{error::Error, io};

use ratatui::{Terminal, backend::CrosstermBackend};

use crate::{
    app_input::clamp_selection,
    app_poll::poll_until_refresh,
    app_terminal::TerminalGuard,
    cli::HtopArgs,
    help,
    interrupt::InterruptFlag,
    metrics::Metrics,
    model::SortMode,
    render::{self, Tab},
    selected_scroll::{keep_selected_visible, selected_data_index},
    signal_menu::SignalMenuState,
    sort_menu,
    tab_sort::normalize_sort_for_tab,
};

pub async fn run(args: HtopArgs) -> Result<(), Box<dyn Error>> {
    let _guard = TerminalGuard::enter()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let mut metrics = Metrics::new();
    let mut tab = Tab::Overview;
    let mut sort = SortMode::Cpu;
    let mut sort_inverted = false;
    let mut sort_menu_open = false;
    let mut sort_cursor = sort;
    let mut sort_header_pressed = None;
    let mut tree = false;
    let mut help_open = false;
    let mut signal_menu: Option<SignalMenuState> = None;
    let mut selected = 0usize;
    let mut detail_scroll = 0usize;
    let mut io_scroll = 0usize;
    let mut network_scroll = 0usize;
    let mut visibility_anchor: Option<(
        Tab,
        SortMode,
        bool,
        Option<String>,
        Option<usize>,
        Option<(u16, u16)>,
    )> = None;
    let mut refresh_ms = args.refresh_ms.max(100);
    let mut filter = String::new();
    let mut filter_input: Option<String> = None;
    let mut search = String::new();
    let mut search_input: Option<String> = None;
    let interrupt = InterruptFlag::new();
    interrupt.listen_for_ctrl_c();

    loop {
        metrics.refresh();
        let active_filter = filter_input.as_deref().unwrap_or(&filter);
        let active_search = search_input.as_deref().unwrap_or(&search);
        sort = normalize_sort_for_tab(tab, sort);
        let snapshot = metrics.snapshot(sort, sort_inverted, active_filter, tree);
        selected = clamp_selection(selected, snapshot.processes.len());
        io_scroll = io_scroll.min(snapshot.io.len().saturating_sub(1));
        network_scroll = network_scroll
            .min(snapshot.networks.len().max(snapshot.sockets.len()).saturating_sub(1));
        let next_visibility_anchor = (
            tab,
            sort,
            sort_inverted,
            snapshot
                .processes
                .get(selected)
                .map(|row| row.pid.clone()),
            selected_data_index(tab, &snapshot, selected),
            terman_common::current_terminal_size().ok(),
        );
        if visibility_anchor.as_ref() != Some(&next_visibility_anchor) {
            keep_selected_visible(
                tab,
                &snapshot,
                selected,
                &mut io_scroll,
                &mut network_scroll,
            );
            visibility_anchor = Some(next_visibility_anchor);
        }
        terminal.draw(|frame| {
            if help_open {
                help::draw(frame);
            } else {
                render::draw(
                    frame,
                    &snapshot,
                    tab,
                    sort,
                    sort_inverted,
                    tree,
                    selected,
                    active_filter,
                    filter_input.is_some(),
                    active_search,
                    search_input.is_some(),
                    detail_scroll,
                    io_scroll,
                    network_scroll,
                    refresh_ms,
                    signal_menu.as_ref(),
                );
                if sort_menu_open {
                    sort_menu::draw(frame, sort_cursor);
                }
            }
        })?;
        if args.once {
            return Ok(());
        }
        if poll_until_refresh(
            &interrupt,
            &mut metrics,
            &mut refresh_ms,
            &mut tab,
            &mut sort,
            &mut sort_inverted,
            &mut sort_menu_open,
            &mut sort_cursor,
            &mut sort_header_pressed,
            &mut tree,
            &mut help_open,
            &mut signal_menu,
            &mut selected,
            &mut detail_scroll,
            &mut io_scroll,
            &mut network_scroll,
            snapshot.processes.as_slice(),
            snapshot.io.as_slice(),
            snapshot.sockets.as_slice(),
            snapshot.cpu_cores.len(),
            &mut filter,
            &mut filter_input,
            &mut search,
            &mut search_input,
        )? {
            return Ok(());
        }
    }
}