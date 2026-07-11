use crate::region_types::ScreenRegionAxis;

use super::region_layout::RegionRect;

pub(super) fn split_rect(
    rect: RegionRect,
    axis: ScreenRegionAxis,
    ratio: u16,
) -> (RegionRect, RegionRect) {
    match axis {
        ScreenRegionAxis::Horizontal => {
            let gap = u16::from(rect.height >= 3);
            let available = rect.height.saturating_sub(gap);
            let first_height = split_extent(available, ratio);
            let second_height = available.saturating_sub(first_height);
            (
                RegionRect { height: first_height, ..rect },
                RegionRect {
                    y: rect.y.saturating_add(first_height).saturating_add(gap),
                    height: second_height,
                    ..rect
                },
            )
        }
        ScreenRegionAxis::Vertical => {
            let gap = u16::from(rect.width >= 3);
            let available = rect.width.saturating_sub(gap);
            let first_width = split_extent(available, ratio);
            let second_width = available.saturating_sub(first_width);
            (
                RegionRect { width: first_width, ..rect },
                RegionRect {
                    x: rect.x.saturating_add(first_width).saturating_add(gap),
                    width: second_width,
                    ..rect
                },
            )
        }
    }
}

fn split_extent(available: u16, ratio: u16) -> u16 {
    if available < 2 { return available / 2; }
    let extent = (u32::from(available) * u32::from(ratio.min(1000)) / 1000) as u16;
    extent.clamp(1, available - 1)
}