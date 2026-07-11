use crate::pane_layout::{PaneDirection, PaneGeometry, PaneRect};

type CandidateScore = (u8, u32, u32, u32);

pub(super) fn neighboring_pane(
    geometry: &PaneGeometry,
    active: u32,
    direction: PaneDirection,
) -> Option<u32> {
    let current = geometry
        .panes
        .iter()
        .find(|(index, _)| *index == active)
        .map(|(_, rect)| *rect)?;
    geometry
        .panes
        .iter()
        .filter(|(index, _)| *index != active)
        .filter_map(|(index, rect)| {
            candidate_score(current, *rect, direction).map(|score| (*index, score))
        })
        .min_by_key(|(index, score)| (*score, *index))
        .map(|(index, _)| index)
}

fn candidate_score(
    current: PaneRect,
    candidate: PaneRect,
    direction: PaneDirection,
) -> Option<CandidateScore> {
    let current_right = edge(current.x, current.cols);
    let current_bottom = edge(current.y, current.rows);
    let candidate_right = edge(candidate.x, candidate.cols);
    let candidate_bottom = edge(candidate.y, candidate.rows);
    let (gap, overlap, center_offset) = match direction {
        PaneDirection::Left if candidate_right <= current.x => (
            u32::from(current.x - candidate_right),
            overlap(current.y, current_bottom, candidate.y, candidate_bottom),
            center_distance(current.y, current.rows, candidate.y, candidate.rows),
        ),
        PaneDirection::Right if candidate.x >= current_right => (
            u32::from(candidate.x - current_right),
            overlap(current.y, current_bottom, candidate.y, candidate_bottom),
            center_distance(current.y, current.rows, candidate.y, candidate.rows),
        ),
        PaneDirection::Up if candidate_bottom <= current.y => (
            u32::from(current.y - candidate_bottom),
            overlap(current.x, current_right, candidate.x, candidate_right),
            center_distance(current.x, current.cols, candidate.x, candidate.cols),
        ),
        PaneDirection::Down if candidate.y >= current_bottom => (
            u32::from(candidate.y - current_bottom),
            overlap(current.x, current_right, candidate.x, candidate_right),
            center_distance(current.x, current.cols, candidate.x, candidate.cols),
        ),
        _ => return None,
    };
    Some((u8::from(overlap == 0), gap, u32::MAX - overlap, center_offset))
}

fn edge(start: u16, size: u16) -> u16 {
    start.saturating_add(size)
}

fn overlap(a_start: u16, a_end: u16, b_start: u16, b_end: u16) -> u32 {
    u32::from(a_end.min(b_end).saturating_sub(a_start.max(b_start)))
}

fn center_distance(a_start: u16, a_size: u16, b_start: u16, b_size: u16) -> u32 {
    let a_center = u32::from(a_start) * 2 + u32::from(a_size);
    let b_center = u32::from(b_start) * 2 + u32::from(b_size);
    a_center.abs_diff(b_center)
}
