use crate::region_types::{
    ScreenRegionAxis, ScreenRegionResize, ScreenRegionResizeAmount, ScreenRegionResizeMode,
};

use super::{
    region_geometry::split_rect,
    region_layout::{RegionNode, RegionRect},
};

pub(super) fn resize_region(
    root: &mut RegionNode,
    focused: u64,
    resize: ScreenRegionResize,
    rect: RegionRect,
) -> bool {
    match resize.mode {
        ScreenRegionResizeMode::Width => resize_axis(root, focused, ScreenRegionAxis::Vertical, resize.amount, rect),
        ScreenRegionResizeMode::Height => resize_axis(root, focused, ScreenRegionAxis::Horizontal, resize.amount, rect),
        ScreenRegionResizeMode::Both => {
            let height = resize_axis(root, focused, ScreenRegionAxis::Horizontal, resize.amount, rect);
            let width = resize_axis(root, focused, ScreenRegionAxis::Vertical, resize.amount, rect);
            height || width
        }
        ScreenRegionResizeMode::Local => deepest_axis(root, focused, rect)
            .is_some_and(|axis| resize_axis(root, focused, axis, resize.amount, rect)),
        ScreenRegionResizeMode::Perpendicular => deepest_axis(root, focused, rect)
            .map(opposite_axis)
            .is_some_and(|axis| resize_axis(root, focused, axis, resize.amount, rect)),
    }
}

fn deepest_axis(node: &RegionNode, focused: u64, rect: RegionRect) -> Option<ScreenRegionAxis> {
    match node {
        RegionNode::Leaf { .. } => None,
        RegionNode::Split { axis, ratio, first, second } => {
            let (first_rect, second_rect) = split_rect(rect, *axis, *ratio);
            if contains(first, focused) {
                deepest_axis(first, focused, first_rect).or(Some(*axis))
            } else if contains(second, focused) {
                deepest_axis(second, focused, second_rect).or(Some(*axis))
            } else {
                None
            }
        }
    }
}

fn resize_axis(
    node: &mut RegionNode,
    focused: u64,
    target: ScreenRegionAxis,
    amount: ScreenRegionResizeAmount,
    rect: RegionRect,
) -> bool {
    let RegionNode::Split { axis, ratio, first, second } = node else { return false; };
    let current_axis = *axis;
    let (first_rect, second_rect) = split_rect(rect, current_axis, *ratio);
    let in_first = contains(first, focused);
    let in_second = !in_first && contains(second, focused);
    if !in_first && !in_second { return false; }
    let resized_child = if in_first {
        resize_axis(first, focused, target, amount, first_rect)
    } else {
        resize_axis(second, focused, target, amount, second_rect)
    };
    if resized_child { return true; }
    if current_axis != target { return false; }
    let available = available_extent(rect, current_axis);
    let Some(next_ratio) = resized_ratio(available, *ratio, in_first, amount) else { return false; };
    *ratio = next_ratio;
    true
}

fn resized_ratio(
    available: u16,
    ratio: u16,
    focused_first: bool,
    amount: ScreenRegionResizeAmount,
) -> Option<u16> {
    if available < 2 { return None; }
    let current_first = split_size(available, ratio);
    let current = if focused_first { current_first } else { available - current_first };
    let maximum = i64::from(available - 1);
    let next = match amount {
        ScreenRegionResizeAmount::Delta(lines) => i64::from(current) + i64::from(lines),
        ScreenRegionResizeAmount::Absolute(lines) => i64::from(lines),
        ScreenRegionResizeAmount::DeltaPercent(percent) => {
            i64::from(current) + i64::from(available) * i64::from(percent) / 100
        }
        ScreenRegionResizeAmount::AbsolutePercent(percent) => {
            i64::from(available) * i64::from(percent) / 100
        }
        ScreenRegionResizeAmount::Equalize => i64::from(available / 2),
        ScreenRegionResizeAmount::Maximum => maximum,
        ScreenRegionResizeAmount::Minimum => 1,
    }
    .clamp(1, maximum) as u16;
    let first = if focused_first { next } else { available - next };
    Some(((u32::from(first) * 1000 + u32::from(available) / 2) / u32::from(available))
        .clamp(1, 999) as u16)
}

fn split_size(available: u16, ratio: u16) -> u16 {
    if available < 2 { return available / 2; }
    let size = (u32::from(available) * u32::from(ratio) / 1000) as u16;
    size.clamp(1, available - 1)
}

fn available_extent(rect: RegionRect, axis: ScreenRegionAxis) -> u16 {
    match axis {
        ScreenRegionAxis::Horizontal => rect.height.saturating_sub(u16::from(rect.height >= 3)),
        ScreenRegionAxis::Vertical => rect.width.saturating_sub(u16::from(rect.width >= 3)),
    }
}

fn contains(node: &RegionNode, focused: u64) -> bool {
    match node {
        RegionNode::Leaf { id, .. } => *id == focused,
        RegionNode::Split { first, second, .. } => contains(first, focused) || contains(second, focused),
    }
}

fn opposite_axis(axis: ScreenRegionAxis) -> ScreenRegionAxis {
    match axis {
        ScreenRegionAxis::Horizontal => ScreenRegionAxis::Vertical,
        ScreenRegionAxis::Vertical => ScreenRegionAxis::Horizontal,
    }
}