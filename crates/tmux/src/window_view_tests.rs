use super::TmuxWindowView;
use crate::pane_layout::SplitDirection;

#[test]
fn zooms_active_pane_without_losing_layout() {
    let mut view = TmuxWindowView::new(120, 40);
    let pane = view
        .reserve_pane(SplitDirection::Vertical)
        .expect("pane should split");

    assert_eq!(view.pane_sizes().len(), 2);
    assert!(view.toggle_zoom(pane.index));
    assert_eq!(view.pane_sizes().len(), 1);
    assert_eq!(view.pane_sizes()[0].index, pane.index);
    assert_eq!(view.pane_sizes()[0].cols, 120);
    assert_eq!(view.pane_sizes()[0].rows, 40);

    assert!(view.toggle_zoom(pane.index));
    assert_eq!(view.pane_sizes().len(), 2);
}