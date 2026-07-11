#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PaneRect {
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) cols: u16,
    pub(crate) rows: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PaneSeparator {
    pub(crate) rect: PaneRect,
}

pub(crate) struct PaneGeometry {
    pub(crate) panes: Vec<(u32, PaneRect)>,
    pub(crate) separators: Vec<PaneSeparator>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PaneLayout {
    Pane(u32),
    Split {
        direction: SplitDirection,
        ratio: u16,
        first: Box<PaneLayout>,
        second: Box<PaneLayout>,
    },
}

impl PaneLayout {
    pub(crate) fn new(index: u32) -> Self {
        Self::Pane(index)
    }

    pub(crate) fn split(
        &mut self,
        target: u32,
        index: u32,
        direction: SplitDirection,
    ) -> bool {
        match self {
            Self::Pane(current) if *current == target => {
                let first = Self::Pane(*current);
                *self = Self::Split {
                    direction,
                    ratio: 500,
                    first: Box::new(first),
                    second: Box::new(Self::Pane(index)),
                };
                true
            }
            Self::Split { first, second, .. } => {
                first.split(target, index, direction)
                    || second.split(target, index, direction)
            }
            Self::Pane(_) => false,
        }
    }

    pub(crate) fn remove(&mut self, target: u32) -> bool {
        if !self.contains(target) {
            return false;
        }
        let current = std::mem::replace(self, Self::Pane(target));
        let Some(next) = current.without(target) else {
            return false;
        };
        *self = next;
        true
    }

    pub(crate) fn contains(&self, target: u32) -> bool {
        match self {
            Self::Pane(index) => *index == target,
            Self::Split { first, second, .. } => {
                first.contains(target) || second.contains(target)
            }
        }
    }


    pub(crate) fn pane_indexes(&self) -> Vec<u32> {
        let mut indexes = Vec::new();
        self.collect_indexes(&mut indexes);
        indexes
    }

    pub(crate) fn geometry(&self, cols: u16, rows: u16) -> PaneGeometry {
        let mut geometry = PaneGeometry {
            panes: Vec::new(),
            separators: Vec::new(),
        };
        self.fill_geometry(
            PaneRect {
                x: 0,
                y: 0,
                cols,
                rows,
            },
            &mut geometry,
        );
        geometry
    }

    pub(crate) fn resize_pane(
        &mut self,
        target: u32,
        cols: Option<u16>,
        rows: Option<u16>,
        area_cols: u16,
        area_rows: u16,
    ) -> bool {
        let area = PaneRect {
            x: 0,
            y: 0,
            cols: area_cols,
            rows: area_rows,
        };
        let width_changed = cols
            .map(|value| self.resize_axis(target, value, SplitDirection::Horizontal, area))
            .unwrap_or(false);
        let height_changed = rows
            .map(|value| self.resize_axis(target, value, SplitDirection::Vertical, area))
            .unwrap_or(false);
        width_changed || height_changed
    }

    fn without(self, target: u32) -> Option<Self> {
        match self {
            Self::Pane(index) if index == target => None,
            Self::Pane(index) => Some(Self::Pane(index)),
            Self::Split {
                direction,
                ratio,
                first,
                second,
            } => match (first.without(target), second.without(target)) {
                (Some(first), Some(second)) => Some(Self::Split {
                    direction,
                    ratio,
                    first: Box::new(first),
                    second: Box::new(second),
                }),
                (Some(layout), None) | (None, Some(layout)) => Some(layout),
                (None, None) => None,
            },
        }
    }


    fn collect_indexes(&self, indexes: &mut Vec<u32>) {
        match self {
            Self::Pane(index) => indexes.push(*index),
            Self::Split { first, second, .. } => {
                first.collect_indexes(indexes);
                second.collect_indexes(indexes);
            }
        }
    }

    fn fill_geometry(&self, area: PaneRect, geometry: &mut PaneGeometry) {
        match self {
            Self::Pane(index) => geometry.panes.push((*index, area)),
            Self::Split {
                direction,
                ratio,
                first,
                second,
            } => {
                let (first_area, separator, second_area) =
                    split_rects(area, *direction, *ratio);
                first.fill_geometry(first_area, geometry);
                if separator.rect.cols > 0 && separator.rect.rows > 0 {
                    geometry.separators.push(separator);
                }
                second.fill_geometry(second_area, geometry);
            }
        }
    }

    fn resize_axis(
        &mut self,
        target: u32,
        desired: u16,
        axis: SplitDirection,
        area: PaneRect,
    ) -> bool {
        let Self::Split {
            direction,
            ratio,
            first,
            second,
        } = self
        else {
            return false;
        };
        let (first_area, _, second_area) = split_rects(area, *direction, *ratio);
        let in_first = first.contains(target);
        let in_second = second.contains(target);
        if !in_first && !in_second {
            return false;
        }
        let nested = if in_first {
            first.resize_axis(target, desired, axis, first_area)
        } else {
            second.resize_axis(target, desired, axis, second_area)
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
        let desired = desired.clamp(1, available - 1);
        let first_size = if in_first { desired } else { available - desired };
        *ratio = ((u32::from(first_size) * 1000) / u32::from(available))
            .clamp(1, 999) as u16;
        true
    }
}

fn split_rects(
    area: PaneRect,
    direction: SplitDirection,
    ratio: u16,
) -> (PaneRect, PaneSeparator, PaneRect) {
    match direction {
        SplitDirection::Horizontal => {
            let usable = area.cols.saturating_sub(1);
            let first_cols = split_size(usable, ratio);
            let second_cols = usable.saturating_sub(first_cols);
            (
                PaneRect { cols: first_cols, ..area },
                PaneSeparator {
                    rect: PaneRect {
                        x: area.x.saturating_add(first_cols),
                        y: area.y,
                        cols: area.cols.min(1),
                        rows: area.rows,
                    },
                },
                PaneRect {
                    x: area.x.saturating_add(first_cols).saturating_add(1),
                    cols: second_cols,
                    ..area
                },
            )
        }
        SplitDirection::Vertical => {
            let usable = area.rows.saturating_sub(1);
            let first_rows = split_size(usable, ratio);
            let second_rows = usable.saturating_sub(first_rows);
            (
                PaneRect { rows: first_rows, ..area },
                PaneSeparator {
                    rect: PaneRect {
                        x: area.x,
                        y: area.y.saturating_add(first_rows),
                        cols: area.cols,
                        rows: area.rows.min(1),
                    },
                },
                PaneRect {
                    y: area.y.saturating_add(first_rows).saturating_add(1),
                    rows: second_rows,
                    ..area
                },
            )
        }
    }
}

fn split_size(total: u16, ratio: u16) -> u16 {
    if total <= 1 {
        return total;
    }
    ((u32::from(total) * u32::from(ratio)) / 1000)
        .clamp(1, u32::from(total - 1)) as u16
}
