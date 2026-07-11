use std::{
    error::Error,
    io,
    sync::{Arc, Mutex},
};

use crate::{
    pane_layout::SplitDirection,
    pane_runtime::{TmuxPaneRuntime, TmuxPaneRuntimeConfig},
    session_core::TmuxSessionBus,
    window_view::{PaneSize, TmuxWindowView},
};

pub(crate) struct TmuxWindowRuntimeConfig {
    pub(crate) session_name: String,
    pub(crate) index: u32,
    pub(crate) name: String,
    pub(crate) command: Option<String>,
    pub(crate) cols: u16,
    pub(crate) rows: u16,
    pub(crate) login_shell: bool,
}

pub(crate) struct TmuxWindowRuntime {
    session_name: String,
    index: u32,
    name: String,
    login_shell: bool,
    panes: Vec<TmuxPaneRuntime>,
    view: Arc<Mutex<TmuxWindowView>>,
    bus: TmuxSessionBus,
}

impl TmuxWindowRuntime {
    pub(crate) fn spawn(
        config: TmuxWindowRuntimeConfig,
        bus: TmuxSessionBus,
    ) -> Result<Self, Box<dyn Error>> {
        let view = Arc::new(Mutex::new(TmuxWindowView::new(config.cols, config.rows)));
        let size = view
            .lock()
            .ok()
            .and_then(|view| view.pane_sizes().into_iter().find(|size| size.index == 0))
            .unwrap_or(PaneSize {
                index: 0,
                cols: config.cols.max(1),
                rows: config.rows.max(1),
            });
        let pane = TmuxPaneRuntime::spawn(
            pane_config(&config, 0, size.cols, size.rows, config.command.clone()),
            view.clone(),
            bus.clone(),
        )?;
        bus.add_window(config.index, config.name.clone());
        let runtime = Self {
            session_name: config.session_name,
            index: config.index,
            name: config.name,
            login_shell: config.login_shell,
            panes: vec![pane],
            view,
            bus,
        };
        runtime.publish_frame();
        Ok(runtime)
    }

    pub(crate) fn index(&self) -> u32 {
        self.index
    }

    pub(crate) fn rename(&mut self, name: String) {
        self.name = name;
    }

    pub(crate) fn write_input(&mut self, bytes: &[u8]) -> io::Result<()> {
        let active = self
            .view
            .lock()
            .map(|view| view.active_pane())
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;
        self.panes
            .iter_mut()
            .find(|pane| pane.index() == active)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "active tmux pane missing"))?
            .write_input(bytes)
    }

    pub(crate) fn split(
        &mut self,
        horizontal: bool,
        command: Option<String>,
    ) -> Result<u32, Box<dyn Error>> {
        let direction = if horizontal {
            SplitDirection::Horizontal
        } else {
            SplitDirection::Vertical
        };
        let reservation = self
            .view
            .lock()
            .ok()
            .and_then(|mut view| view.reserve_pane(direction))
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "unable to split active tmux pane")
            })?;
        let config = TmuxPaneRuntimeConfig {
            session_name: self.session_name.clone(),
            window_index: self.index,
            window_name: self.name.clone(),
            pane_index: reservation.index,
            command,
            cols: reservation.cols,
            rows: reservation.rows,
            login_shell: self.login_shell,
        };
        match TmuxPaneRuntime::spawn(config, self.view.clone(), self.bus.clone()) {
            Ok(pane) => {
                let index = pane.index();
                self.panes.push(pane);
                self.resize_from_view();
                self.publish_frame();
                Ok(index)
            }
            Err(err) => {
                if let Ok(mut view) = self.view.lock() {
                    let _ = view.remove_pane(reservation.index);
                }
                self.publish_frame();
                Err(err)
            }
        }
    }

    pub(crate) fn select_pane(&mut self, index: u32) -> bool {
        let selected = self
            .view
            .lock()
            .map(|mut view| view.select_pane(index))
            .unwrap_or(false);
        if selected {
            self.publish_frame();
        }
        selected
    }

    pub(crate) fn kill_pane(&mut self, index: u32) -> bool {
        let Some(pane) = self.panes.iter_mut().find(|pane| pane.index() == index) else {
            return false;
        };
        pane.kill();
        true
    }

    pub(crate) fn resize_pane(
        &mut self,
        index: u32,
        cols: Option<u16>,
        rows: Option<u16>,
    ) -> bool {
        let changed = self
            .view
            .lock()
            .map(|mut view| view.resize_pane(index, cols, rows))
            .unwrap_or(false);
        if changed {
            self.resize_from_view();
            self.publish_frame();
        }
        changed
    }

    pub(crate) fn resize(&mut self, cols: u16, rows: u16) {
        if let Ok(mut view) = self.view.lock() {
            view.resize(cols, rows);
        }
        self.resize_from_view();
        self.publish_frame();
    }

    pub(crate) fn try_exit_code(&mut self) -> io::Result<Option<i32>> {
        let mut exited = None;
        for (position, pane) in self.panes.iter_mut().enumerate() {
            if let Some(code) = pane.try_exit_code()? {
                exited = Some((position, code));
                break;
            }
        }
        let Some((position, code)) = exited else {
            return Ok(None);
        };
        let mut pane = self.panes.remove(position);
        let index = pane.index();
        pane.join_output();
        if self.panes.is_empty() {
            return Ok(Some(code));
        }
        if let Ok(mut view) = self.view.lock() {
            let _ = view.remove_pane(index);
        }
        self.resize_from_view();
        self.publish_frame();
        Ok(None)
    }

    pub(crate) fn kill(&mut self) {
        for pane in &mut self.panes {
            pane.kill();
        }
    }

    pub(crate) fn join_output(&mut self) {
        for pane in &mut self.panes {
            pane.join_output();
        }
    }

    fn resize_from_view(&self) {
        let sizes = self
            .view
            .lock()
            .map(|view| view.pane_sizes())
            .unwrap_or_default();
        for size in sizes {
            if let Some(pane) = self.panes.iter().find(|pane| pane.index() == size.index) {
                pane.resize(size.cols, size.rows);
            }
        }
    }

    fn publish_frame(&self) {
        let rendered = self.view.lock().ok().map(|view| view.render());
        if let Some((active_pane, rendered)) = rendered {
            self.bus.publish_window_frame(
                self.index,
                rendered.frame,
                active_pane,
                rendered.captures,
            );
        }
    }
}

fn pane_config(
    config: &TmuxWindowRuntimeConfig,
    pane_index: u32,
    cols: u16,
    rows: u16,
    command: Option<String>,
) -> TmuxPaneRuntimeConfig {
    TmuxPaneRuntimeConfig {
        session_name: config.session_name.clone(),
        window_index: config.index,
        window_name: config.name.clone(),
        pane_index,
        command,
        cols,
        rows,
        login_shell: config.login_shell,
    }
}
