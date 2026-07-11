use super::TmuxWindowView;
use crate::pane_layout::{PaneDirection, SplitDirection};

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
#[test]
fn moves_the_nearest_split_boundary_in_the_requested_direction() {
    let mut view = TmuxWindowView::new(120, 40);
    let pane = view
        .reserve_pane(SplitDirection::Horizontal)
        .expect("pane should split");
    let initial = view.pane_sizes().into_iter()
        .find(|size| size.index == pane.index).expect("pane size").cols;

    assert!(view.resize_pane_direction(pane.index, PaneDirection::Left, 5));
    let expanded = view.pane_sizes().into_iter()
        .find(|size| size.index == pane.index).expect("pane size").cols;
    assert!(expanded > initial);

    assert!(view.resize_pane_direction(pane.index, PaneDirection::Right, 5));
    let restored = view.pane_sizes().into_iter()
        .find(|size| size.index == pane.index).expect("pane size").cols;
    assert!(restored < expanded);
}