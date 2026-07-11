use std::{collections::HashSet, error::Error, io};

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
    process_tree::ProcessTreeState,
    render::{self, Tab},
    selected_scroll::{keep_selected_visible, selected_data_index},
    setup_menu::SetupMenuState,
    signal_menu::SignalMenuState,
    sort_menu,
    tab_sort::normalize_sort_for_tab,
    user_filter::UserFilterState,
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
    let mut user_filter = UserFilterState::default();
    let mut tree = false;
    let mut tree_state = ProcessTreeState::default();
    let mut help_open = false;
    let mut setup_menu = SetupMenuState::default();
    let mut signal_menu: Option<SignalMenuState> = None;
    let mut selected = 0usize;
    let mut followed_pid: Option<String> = None;
    let mut tagged_pids: HashSet<String> = HashSet::new();
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
        let snapshot = metrics.snapshot(
            sort,
            sort_inverted,
            active_filter,
            user_filter.selected(),
            tree,
            &tree_state,
        );
        let followed_index = followed_pid.as_deref().and_then(|pid| {
            snapshot.processes.iter().position(|process| process.pid == pid)
        });
        if followed_pid.is_some() {
            if let Some(index) = followed_index {
                selected = index;
            } else {
                followed_pid = None;
            }
        }
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
                tagged_pids.retain(|pid| metrics.process_exists(pid));
        render::draw(
                    frame,
                    &snapshot,
                    tab,
                    sort,
                    sort_inverted,
                    tree,
                    user_filter.selected(),
                    selected,
                    active_filter,
                    filter_input.is_some(),
                    active_search,
                    search_input.is_some(),
                    detail_scroll,
                    io_scroll,
                    network_scroll,
                    refresh_ms,
                    followed_pid.as_deref(),
                    signal_menu.as_ref(),
                    &tagged_pids,
                );
                if sort_menu_open {
                    sort_menu::draw(frame, sort_cursor);
                }
                setup_menu.draw(frame, refresh_ms, tree, sort_inverted);
                user_filter.draw(frame);
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
            &mut setup_menu,
            &mut user_filter,
            &mut tree,
            &mut tree_state,
            &mut help_open,
            &mut signal_menu,
            &mut selected,
            &mut followed_pid,
            &mut detail_scroll,
            &mut io_scroll,
            &mut network_scroll,
            snapshot.processes.as_slice(),
            snapshot.process_users.as_slice(),
            snapshot.io.as_slice(),
            snapshot.sockets.as_slice(),
            snapshot.cpu_cores.len(),
            &mut filter,
            &mut filter_input,
            &mut search,
            &mut search_input,
            &mut tagged_pids,
        )? {
            return Ok(());
        }
    }
}
