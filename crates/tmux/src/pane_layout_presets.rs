use super::{PaneLayout, SplitDirection};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PaneLayoutPreset {
    EvenHorizontal,
    EvenVertical,
    MainHorizontal,
    MainVertical,
    Tiled,
}

impl PaneLayoutPreset {
    pub(crate) fn next(self) -> Self {
        match self {
            Self::EvenHorizontal => Self::EvenVertical,
            Self::EvenVertical => Self::MainHorizontal,
            Self::MainHorizontal => Self::MainVertical,
            Self::MainVertical => Self::Tiled,
            Self::Tiled => Self::EvenHorizontal,
        }
    }
}

impl PaneLayout {
    pub(crate) fn from_preset(
        indexes: &[u32],
        preset: PaneLayoutPreset,
    ) -> Option<Self> {
        match preset {
            PaneLayoutPreset::EvenHorizontal => {
                even_layout(indexes, SplitDirection::Horizontal)
            }
            PaneLayoutPreset::EvenVertical => {
                even_layout(indexes, SplitDirection::Vertical)
            }
            PaneLayoutPreset::MainHorizontal => main_layout(
                indexes,
                SplitDirection::Vertical,
                SplitDirection::Horizontal,
            ),
            PaneLayoutPreset::MainVertical => main_layout(
                indexes,
                SplitDirection::Horizontal,
                SplitDirection::Vertical,
            ),
            PaneLayoutPreset::Tiled => tiled_layout(indexes),
        }
    }
}

fn even_layout(
    indexes: &[u32],
    direction: SplitDirection,
) -> Option<PaneLayout> {
    combine_equal(
        indexes
            .iter()
            .copied()
            .map(PaneLayout::Pane)
            .collect(),
        direction,
    )
}

fn main_layout(
    indexes: &[u32],
    primary: SplitDirection,
    secondary: SplitDirection,
) -> Option<PaneLayout> {
    let (&main, rest) = indexes.split_first()?;
    if rest.is_empty() {
        return Some(PaneLayout::Pane(main));
    }
    Some(PaneLayout::Split {
        direction: primary,
        ratio: 600,
        first: Box::new(PaneLayout::Pane(main)),
        second: Box::new(even_layout(rest, secondary)?),
    })
}

fn tiled_layout(indexes: &[u32]) -> Option<PaneLayout> {
    if indexes.is_empty() {
        return None;
    }
    let columns = tiled_columns(indexes.len());
    let rows = indexes
        .chunks(columns)
        .filter_map(|row| {
            even_layout(row, SplitDirection::Horizontal)
        })
        .collect();
    combine_equal(rows, SplitDirection::Vertical)
}

fn tiled_columns(count: usize) -> usize {
    let mut columns = 1usize;
    while columns.saturating_mul(columns) < count {
        columns = columns.saturating_add(1);
    }
    columns
}

fn combine_equal(
    mut layouts: Vec<PaneLayout>,
    direction: SplitDirection,
) -> Option<PaneLayout> {
    if layouts.is_empty() {
        return None;
    }
    if layouts.len() == 1 {
        return layouts.pop();
    }
    let count = layouts.len();
    let first = layouts.remove(0);
    let ratio = (1000usize / count).clamp(1, 999) as u16;
    Some(PaneLayout::Split {
        direction,
        ratio,
        first: Box::new(first),
        second: Box::new(combine_equal(layouts, direction)?),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_preset_preserves_pane_indexes() {
        let indexes = [0, 2, 5, 8, 9];
        let presets = [
            PaneLayoutPreset::EvenHorizontal,
            PaneLayoutPreset::EvenVertical,
            PaneLayoutPreset::MainHorizontal,
            PaneLayoutPreset::MainVertical,
            PaneLayoutPreset::Tiled,
        ];
        for preset in presets {
            let layout =
                PaneLayout::from_preset(&indexes, preset).unwrap();
            assert_eq!(layout.pane_indexes(), indexes);
        }
    }

    #[test]
    fn tiled_layout_uses_multiple_axes() {
        let layout = PaneLayout::from_preset(
            &[0, 1, 2, 3],
            PaneLayoutPreset::Tiled,
        )
        .unwrap();
        let geometry = layout.geometry(80, 24);
        assert_eq!(geometry.panes.len(), 4);
        assert!(geometry.separators.len() >= 3);
    }
}