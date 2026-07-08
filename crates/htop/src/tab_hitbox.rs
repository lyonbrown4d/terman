use unicode_width::UnicodeWidthStr;

use crate::render::Tab;

const TAB_ROW: u16 = 3;
const TAB_PADDING: u16 = 2;
const TAB_GAP: u16 = 1;

type TabLabel = (Tab, String);

pub(crate) fn tab_at(column: u16, row: u16) -> Option<Tab> {
    if row != TAB_ROW {
        return None;
    }
    tab_labels()
        .into_iter()
        .scan(0u16, |offset, (tab, label)| {
            let start = *offset;
            let width = tab_width(label.as_str());
            *offset = offset.saturating_add(width).saturating_add(TAB_GAP);
            Some((tab, start, width))
        })
        .find_map(|(tab, start, width)| column_in_span(column, start, width).then_some(tab))
}

fn tab_labels() -> [TabLabel; 4] {
    [
        (Tab::Overview, terman_common::builtin_htop_tab_overview_hint()),
        (Tab::Processes, terman_common::builtin_htop_tab_processes_hint()),
        (Tab::Io, terman_common::builtin_htop_tab_io_hint()),
        (Tab::Network, terman_common::builtin_htop_tab_network_hint()),
    ]
}

fn tab_width(label: &str) -> u16 {
    let width = UnicodeWidthStr::width(label).saturating_add(TAB_PADDING as usize);
    width.min(u16::MAX as usize) as u16
}

fn column_in_span(column: u16, start: u16, width: u16) -> bool {
    column >= start && column < start.saturating_add(width)
}

#[cfg(test)]
mod tests {
    use super::tab_at;
    use crate::render::Tab;

    #[test]
    fn maps_tab_clicks() {
        assert_eq!(tab_at(1, 3), Some(Tab::Overview));
        assert_eq!(tab_at(0, 0), None);
    }
}
