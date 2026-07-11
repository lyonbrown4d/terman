#[path = "pane_navigation.rs"]
mod navigation;

use crate::{
    pane_layout::{PaneDirection, PaneGeometry, PaneLayout, PaneRect, SplitDirection},
    terminal_frame::{PaneRender, RenderedTerminal, render_terminal},
};

pub(crate) struct PaneReservation {
    pub(crate) index: u32,
    pub(crate) cols: u16,
    pub(crate) rows: u16,
}

#[derive(Clone, Copy)]
pub(crate) struct PaneSize {
    pub(crate) index: u32,
    pub(crate) cols: u16,
    pub(crate) rows: u16,
}

struct PaneTerminal {
    index: u32,
    parser: vt100::Parser,
}

pub(crate) struct TmuxWindowView {
    layout: PaneLayout,
    panes: Vec<PaneTerminal>,
    active_pane: u32,
    next_pane: u32,
    cols: u16,
    rows: u16,
    zoomed: bool,
}

impl TmuxWindowView {
    pub(crate) fn new(cols: u16, rows: u16) -> Self {
        let cols = cols.max(1);
        let rows = rows.max(1);
        Self {
            layout: PaneLayout::new(0),
            panes: vec![PaneTerminal {
                index: 0,
                parser: vt100::Parser::new(rows, cols, 10_000),
            }],
            active_pane: 0,
            next_pane: 1,
            cols,
            rows,
            zoomed: false,
        }
    }

    pub(crate) fn active_pane(&self) -> u32 {
        self.active_pane
    }

    pub(crate) fn reserve_pane(
        &mut self,
        direction: SplitDirection,
    ) -> Option<PaneReservation> {
        let index = self.next_pane;
        if !self.layout.split(self.active_pane, index, direction) {
            return None;
        }
        self.zoomed = false;
        self.next_pane = self.next_pane.saturating_add(1);
        self.panes.push(PaneTerminal {
            index,
            parser: vt100::Parser::new(1, 1, 10_000),
        });
        self.active_pane = index;
        self.resize(self.cols, self.rows);
        self.pane_sizes()
            .into_iter()
            .find(|size| size.index == index)
            .map(|size| PaneReservation {
                index,
                cols: size.cols,
                rows: size.rows,
            })
    }

    pub(crate) fn remove_pane(&mut self, index: u32) -> bool {
        if self.panes.len() <= 1 || !self.layout.remove(index) {
            return false;
        }
        self.panes.retain(|pane| pane.index != index);
        if self.active_pane == index {
            self.active_pane = self.layout.pane_indexes().into_iter().next().unwrap_or(0);
        }
        if self.panes.len() <= 1 {
            self.zoomed = false;
        }
        self.resize(self.cols, self.rows);
        true
    }

    pub(crate) fn select_pane(&mut self, index: u32) -> bool {
        if !self.panes.iter().any(|pane| pane.index == index) {
            return false;
        }
        self.active_pane = index;
        true
    }

    pub(crate) fn select_pane_direction(&mut self, direction: PaneDirection) -> bool {
        let geometry = self.layout.geometry(self.cols, self.rows);
        let Some(index) = navigation::neighboring_pane(&geometry, self.active_pane, direction) else {
            return false;
        };
        self.select_pane(index)
    }

    pub(crate) fn swap_panes(&mut self, source: u32, target: u32) -> bool {
        if !self.layout.swap_panes(source, target) {
            return false;
        }
        self.resize(self.cols, self.rows);
        true
    }

    pub(crate) fn toggle_zoom(&mut self, index: u32) -> bool {
        if self.panes.len() <= 1 || !self.select_pane(index) {
            return false;
        }
        self.zoomed = !self.zoomed;
        self.resize(self.cols, self.rows);
        true
    }

    pub(crate) fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols.max(1);
        self.rows = rows.max(1);
        for size in self.pane_sizes() {
            if let Some(pane) = self.panes.iter_mut().find(|pane| pane.index == size.index) {
                pane.parser
                    .screen_mut()
                    .set_size(size.rows.max(1), size.cols.max(1));
            }
        }
    }

    pub(crate) fn resize_pane(
        &mut self,
        index: u32,
        cols: Option<u16>,
        rows: Option<u16>,
    ) -> bool {
        if !self.layout.resize_pane(index, cols, rows, self.cols, self.rows) {
            return false;
        }
        self.resize(self.cols, self.rows);
        true
    }

    pub(crate) fn pane_sizes(&self) -> Vec<PaneSize> {
        self.geometry()
            .panes
            .into_iter()
            .map(|(index, rect)| PaneSize {
                index,
                cols: rect.cols.max(1),
                rows: rect.rows.max(1),
            })
            .collect()
    }

    pub(crate) fn process_output(
        &mut self,
        index: u32,
        bytes: &[u8],
    ) -> Option<(u32, RenderedTerminal)> {
        let pane = self.panes.iter_mut().find(|pane| pane.index == index)?;
        pane.parser.process(bytes);
        Some(self.render())
    }

    pub(crate) fn render(&self) -> (u32, RenderedTerminal) {
        let geometry = self.geometry();
        let panes = geometry
            .panes
            .iter()
            .filter_map(|(index, rect)| self.pane_render(*index, *rect))
            .collect::<Vec<_>>();
        let mut rendered = render_terminal(&panes, &geometry.separators);
        rendered.captures = self
            .layout
            .pane_indexes()
            .into_iter()
            .filter_map(|index| {
                self.panes.iter().find(|pane| pane.index == index).map(|pane| {
                    (index, pane.parser.screen().contents().into_bytes())
                })
            })
            .collect();
        (self.active_pane, rendered)
    }

    fn geometry(&self) -> PaneGeometry {
        if self.zoomed {
            PaneGeometry {
                panes: vec![(
                    self.active_pane,
                    PaneRect {
                        x: 0,
                        y: 0,
                        cols: self.cols,
                        rows: self.rows,
                    },
                )],
                separators: Vec::new(),
            }
        } else {
            self.layout.geometry(self.cols, self.rows)
        }
    }

    fn pane_render(&self, index: u32, rect: PaneRect) -> Option<PaneRender<'_>> {
        self.panes
            .iter()
            .find(|pane| pane.index == index)
            .map(|pane| PaneRender {
                index,
                rect,
                screen: pane.parser.screen(),
                active: index == self.active_pane,
            })
    }
}

#[cfg(test)]
#[path = "window_view_tests.rs"]
mod tests;
