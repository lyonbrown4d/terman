use super::{PaneDirection, PaneLayout, PaneRect, SplitDirection, split_rects};

impl PaneLayout {
    pub(crate) fn resize_pane_direction(
        &mut self,
        target: u32,
        movement: PaneDirection,
        adjustment: u16,
        area_cols: u16,
        area_rows: u16,
    ) -> bool {
        self.resize_boundary(
            target,
            movement,
            adjustment.max(1),
            PaneRect { x: 0, y: 0, cols: area_cols, rows: area_rows },
        )
    }

    fn resize_boundary(
        &mut self,
        target: u32,
        movement: PaneDirection,
        adjustment: u16,
        area: PaneRect,
    ) -> bool {
        let Self::Split { direction, ratio, first, second } = self else {
            return false;
        };
        let (first_area, _, second_area) = split_rects(area, *direction, *ratio);
        let in_first = first.contains(target);
        let in_second = second.contains(target);
        if !in_first && !in_second {
            return false;
        }
        let nested = if in_first {
            first.resize_boundary(target, movement, adjustment, first_area)
        } else {
            second.resize_boundary(target, movement, adjustment, second_area)
        };
        let axis = match movement {
            PaneDirection::Left | PaneDirection::Right => SplitDirection::Horizontal,
            PaneDirection::Up | PaneDirection::Down => SplitDirection::Vertical,
        };
        if nested || *direction != axis {
            return nested;
        }
        let available = match axis {
            SplitDirection::Horizontal => area.cols.saturating_sub(1),
            SplitDirection::Vertical => area.rows.saturating_sub(1),
        };
        if available < 2 {
            return false;
        }
        let first_size = match axis {
            SplitDirection::Horizontal => first_area.cols,
            SplitDirection::Vertical => first_area.rows,
        };
        let desired = match movement {
            PaneDirection::Left | PaneDirection::Up => first_size.saturating_sub(adjustment),
            PaneDirection::Right | PaneDirection::Down => first_size.saturating_add(adjustment),
        }
        .clamp(1, available - 1);
        if desired == first_size {
            return false;
        }
        let next_ratio = (u32::from(desired) * 1000 + u32::from(available) - 1)
            / u32::from(available);
        *ratio = next_ratio.clamp(1, 999) as u16;
        true
    }
}