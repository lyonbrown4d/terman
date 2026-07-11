use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    attach_status::{query_status_line, render_status_line},
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
};

const PANE_CHOOSER_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Default)]
pub(crate) struct PaneChooserState {
    active: Option<ActivePaneChooser>,
}

struct ActivePaneChooser {
    window: u32,
    choices: Vec<PaneChoice>,
    status: String,
    opened: Instant,
}

struct PaneChoice {
    label: u32,
    pane: u32,
}

impl PaneChooserState {
    pub(crate) fn open(&mut self, endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
        let (window, active_pane, pane_indexes) = query_panes(endpoint)?;
        let choices = pane_indexes
            .into_iter()
            .take(10)
            .enumerate()
            .map(|(label, pane)| PaneChoice {
                label: label as u32,
                pane,
            })
            .collect::<Vec<_>>();
        let panes = choices
            .iter()
            .map(|choice| {
                let marker = if choice.pane == active_pane { "*" } else { "" };
                format!("{}:#{}{}", choice.label, choice.pane, marker)
            })
            .collect::<Vec<_>>()
            .join(" ");
        let status = terman_common::builtin_tmux_pane_chooser_hint(&panes);
        render_status_line(&status)?;
        self.active = Some(ActivePaneChooser {
            window,
            choices,
            status,
            opened: Instant::now(),
        });
        Ok(())
    }

    pub(crate) fn handle_key(
        &mut self,
        endpoint: &TmuxIpcEndpoint,
        key: &KeyEvent,
    ) -> io::Result<bool> {
        let Some(active) = self.active.as_ref() else {
            return Ok(false);
        };
        if active.expired() {
            self.active = None;
            return Ok(false);
        }
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.close(endpoint)?;
            }
            KeyCode::Char(ch)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                    && ch.is_ascii_digit() =>
            {
                let label = ch.to_digit(10).unwrap_or_default();
                let selection = active
                    .choices
                    .iter()
                    .find(|choice| choice.label == label)
                    .map(|choice| (active.window, choice.pane));
                if let Some((window, pane)) = selection {
                    select_pane(endpoint, window, pane)?;
                    self.close(endpoint)?;
                }
            }
            _ => {}
        }
        Ok(true)
    }

    pub(crate) fn status_override(&self) -> Option<String> {
        self.active
            .as_ref()
            .filter(|active| !active.expired())
            .map(|active| active.status.clone())
    }

    pub(crate) fn is_active(&self) -> bool {
        self.active
            .as_ref()
            .is_some_and(|active| !active.expired())
    }

    fn close(&mut self, endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
        self.active = None;
        render_status_line(&query_status_line(endpoint)?)
    }
}

impl ActivePaneChooser {
    fn expired(&self) -> bool {
        self.opened.elapsed() >= PANE_CHOOSER_TIMEOUT
    }
}

fn query_panes(endpoint: &TmuxIpcEndpoint) -> io::Result<(u32, u32, Vec<u32>)> {
    match request_endpoint_response(endpoint, TmuxIpcRequest::PaneInfo { window: None })? {
        TmuxIpcResponse::Panes {
            window_index,
            active_pane,
            pane_indexes,
            ..
        } => Ok((window_index, active_pane, pane_indexes)),
        TmuxIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, reason))
        }
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")),
        )),
    }
}

fn select_pane(endpoint: &TmuxIpcEndpoint, window: u32, pane: u32) -> io::Result<()> {
    match request_endpoint_response(
        endpoint,
        TmuxIpcRequest::SelectPane {
            window: Some(window),
            pane: Some(pane),
        },
    )? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, reason))
        }
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_tmux_unexpected_response_hint(&format!("{response:?}")),
        )),
    }
}