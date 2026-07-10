use crate::{model::SortMode, render::Tab};

pub(crate) fn normalize_sort_for_tab(tab: Tab, sort: SortMode) -> SortMode {
    match tab {
        Tab::Io => io_sort(sort),
        Tab::Network => network_sort(sort),
        Tab::Overview | Tab::Processes => sort,
    }
}

fn io_sort(sort: SortMode) -> SortMode {
    match sort {
        SortMode::Pid | SortMode::Io | SortMode::Name => sort,
        _ => SortMode::Io,
    }
}

fn network_sort(sort: SortMode) -> SortMode {
    match sort {
        SortMode::State | SortMode::Pid | SortMode::Name => sort,
        _ => SortMode::Name,
    }
}