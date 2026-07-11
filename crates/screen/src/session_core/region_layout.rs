use crate::region_types::{ScreenRegionAxis, ScreenRegionFocus};

#[derive(Clone)]
enum RegionNode {
    Leaf {
        id: u64,
        window_index: usize,
    },
    Split {
        axis: ScreenRegionAxis,
        first: Box<RegionNode>,
        second: Box<RegionNode>,
    },
}

#[derive(Clone, Copy)]
pub(super) struct RegionRect {
    pub(super) x: u16,
    pub(super) y: u16,
    pub(super) width: u16,
    pub(super) height: u16,
}

pub(super) struct ScreenRegionView {
    pub(super) window_index: usize,
    pub(super) rect: RegionRect,
    pub(super) focused: bool,
}

pub(super) struct ScreenRegionLayout {
    root: RegionNode,
    focused: u64,
    next_id: u64,
}

impl ScreenRegionLayout {
    pub(super) fn new(window_index: usize) -> Self {
        Self {
            root: RegionNode::Leaf { id: 0, window_index },
            focused: 0,
            next_id: 1,
        }
    }

    pub(super) fn len(&self) -> usize {
        let mut leaves = Vec::new();
        collect_leaves(&self.root, &mut leaves);
        leaves.len()
    }

    pub(super) fn contains_window(&self, window_index: usize) -> bool {
        let mut leaves = Vec::new();
        collect_leaves(&self.root, &mut leaves);
        leaves.iter().any(|(_, index)| *index == window_index)
    }

    pub(super) fn split(&mut self, axis: ScreenRegionAxis) -> bool {
        let Some(window_index) = window_for(&self.root, self.focused) else {
            return false;
        };
        let new_id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        split_leaf(&mut self.root, self.focused, new_id, window_index, axis)
    }

    pub(super) fn focus(&mut self, target: ScreenRegionFocus) -> Option<usize> {
        let mut leaves = Vec::new();
        collect_leaves(&self.root, &mut leaves);
        let position = leaves.iter().position(|(id, _)| *id == self.focused)?;
        let next = match target {
            ScreenRegionFocus::Next => (position + 1) % leaves.len(),
            ScreenRegionFocus::Previous => {
                position.checked_sub(1).unwrap_or(leaves.len().saturating_sub(1))
            }
            ScreenRegionFocus::First => 0,
            ScreenRegionFocus::Last => leaves.len().saturating_sub(1),
        };
        self.focused = leaves[next].0;
        Some(leaves[next].1)
    }

    pub(super) fn remove_focused(&mut self) -> Option<usize> {
        let mut leaves = Vec::new();
        collect_leaves(&self.root, &mut leaves);
        if leaves.len() <= 1 {
            return None;
        }
        let position = leaves.iter().position(|(id, _)| *id == self.focused)?;
        let replacement = if position + 1 < leaves.len() {
            leaves[position + 1]
        } else {
            leaves[position - 1]
        };
        let placeholder = RegionNode::Leaf {
            id: replacement.0,
            window_index: replacement.1,
        };
        let root = std::mem::replace(&mut self.root, placeholder);
        self.root = remove_leaf(root, self.focused)?;
        self.focused = replacement.0;
        Some(replacement.1)
    }

    pub(super) fn keep_focused_only(&mut self) -> Option<usize> {
        if self.len() <= 1 {
            return None;
        }
        let window_index = window_for(&self.root, self.focused)?;
        self.root = RegionNode::Leaf {
            id: self.focused,
            window_index,
        };
        Some(window_index)
    }

    pub(super) fn select_window(&mut self, window_index: usize) {
        set_focused_window(&mut self.root, self.focused, window_index);
    }

    pub(super) fn swap_windows(&mut self, left: usize, right: usize) {
        map_windows(&mut self.root, &mut |index| {
            if index == left {
                right
            } else if index == right {
                left
            } else {
                index
            }
        });
    }

    pub(super) fn replace_window(&mut self, source: usize, replacement: usize) {
        map_windows(&mut self.root, &mut |index| {
            if index == source { replacement } else { index }
        });
    }

    pub(super) fn views(&self, rows: u16, cols: u16) -> Vec<ScreenRegionView> {
        let mut views = Vec::new();
        layout_node(
            &self.root,
            RegionRect { x: 0, y: 0, width: cols, height: rows },
            self.focused,
            &mut views,
        );
        views
    }
}

fn collect_leaves(node: &RegionNode, leaves: &mut Vec<(u64, usize)>) {
    match node {
        RegionNode::Leaf { id, window_index } => leaves.push((*id, *window_index)),
        RegionNode::Split { first, second, .. } => {
            collect_leaves(first, leaves);
            collect_leaves(second, leaves);
        }
    }
}

fn window_for(node: &RegionNode, target: u64) -> Option<usize> {
    match node {
        RegionNode::Leaf { id, window_index } => (*id == target).then_some(*window_index),
        RegionNode::Split { first, second, .. } => {
            window_for(first, target).or_else(|| window_for(second, target))
        }
    }
}

fn split_leaf(
    node: &mut RegionNode,
    target: u64,
    new_id: u64,
    window_index: usize,
    axis: ScreenRegionAxis,
) -> bool {
    match node {
        RegionNode::Leaf { id, .. } if *id == target => {
            let first = node.clone();
            *node = RegionNode::Split {
                axis,
                first: Box::new(first),
                second: Box::new(RegionNode::Leaf { id: new_id, window_index }),
            };
            true
        }
        RegionNode::Leaf { .. } => false,
        RegionNode::Split { first, second, .. } => {
            split_leaf(first, target, new_id, window_index, axis)
                || split_leaf(second, target, new_id, window_index, axis)
        }
    }
}

fn remove_leaf(node: RegionNode, target: u64) -> Option<RegionNode> {
    match node {
        RegionNode::Leaf { id, .. } if id == target => None,
        RegionNode::Leaf { .. } => Some(node),
        RegionNode::Split { axis, first, second } => {
            let first = remove_leaf(*first, target);
            let second = remove_leaf(*second, target);
            match (first, second) {
                (Some(first), Some(second)) => Some(RegionNode::Split {
                    axis,
                    first: Box::new(first),
                    second: Box::new(second),
                }),
                (Some(node), None) | (None, Some(node)) => Some(node),
                (None, None) => None,
            }
        }
    }
}

fn set_focused_window(node: &mut RegionNode, target: u64, window_index: usize) -> bool {
    match node {
        RegionNode::Leaf { id, window_index: current } if *id == target => {
            *current = window_index;
            true
        }
        RegionNode::Leaf { .. } => false,
        RegionNode::Split { first, second, .. } => {
            set_focused_window(first, target, window_index)
                || set_focused_window(second, target, window_index)
        }
    }
}

fn map_windows(node: &mut RegionNode, map: &mut impl FnMut(usize) -> usize) {
    match node {
        RegionNode::Leaf { window_index, .. } => *window_index = map(*window_index),
        RegionNode::Split { first, second, .. } => {
            map_windows(first, map);
            map_windows(second, map);
        }
    }
}

fn layout_node(
    node: &RegionNode,
    rect: RegionRect,
    focused: u64,
    views: &mut Vec<ScreenRegionView>,
) {
    match node {
        RegionNode::Leaf { id, window_index } => views.push(ScreenRegionView {
            window_index: *window_index,
            rect,
            focused: *id == focused,
        }),
        RegionNode::Split { axis, first, second } => {
            let (first_rect, second_rect) = split_rect(rect, *axis);
            layout_node(first, first_rect, focused, views);
            layout_node(second, second_rect, focused, views);
        }
    }
}

fn split_rect(rect: RegionRect, axis: ScreenRegionAxis) -> (RegionRect, RegionRect) {
    match axis {
        ScreenRegionAxis::Horizontal => {
            let gap = u16::from(rect.height >= 3);
            let available = rect.height.saturating_sub(gap);
            let first_height = available / 2;
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
            let first_width = available / 2;
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