use crate::pane_layout::PaneLayout;

impl PaneLayout {
    pub(crate) fn swap_panes(&mut self, source: u32, target: u32) -> bool {
        if source == target {
            return self.contains(source);
        }
        if !self.contains(source) || !self.contains(target) {
            return false;
        }
        self.swap_indexes(source, target);
        true
    }

    fn swap_indexes(&mut self, source: u32, target: u32) {
        match self {
            Self::Pane(index) if *index == source => *index = target,
            Self::Pane(index) if *index == target => *index = source,
            Self::Split { first, second, .. } => {
                first.swap_indexes(source, target);
                second.swap_indexes(source, target);
            }
            Self::Pane(_) => {}
        }
    }
}