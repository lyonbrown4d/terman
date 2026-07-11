use std::io;

use crossterm::event::KeyEvent;

use crate::{
    attach_keys::is_key_press,
    attach_prompt::{PromptAction, edit_prompt},
    attach_status::{query_status_line, render_status_line},
    attach_window::select_window,
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FindWindowResult {
    Inactive,
    Handled,
    Selected(u32),
}

#[derive(Default)]
pub(crate) struct FindWindowState {
    input: Option<String>,
    message: Option<String>,
}

impl FindWindowState {
    pub(crate) fn begin(&mut self) -> io::Result<()> {
        self.message = None;
        self.input = Some(String::new());
        self.render()
    }

    pub(crate) fn is_active(&self) -> bool {
        self.input.is_some()
    }

    pub(crate) fn status_override(&self) -> Option<String> {
        self.input
            .as_deref()
            .map(terman_common::builtin_tmux_find_prompt_hint)
            .or_else(|| self.message.clone())
    }

    pub(crate) fn handle_key(
        &mut self,
        endpoint: &TmuxIpcEndpoint,
        key: &KeyEvent,
    ) -> io::Result<FindWindowResult> {
        if !is_key_press(key) {
            return Ok(if self.is_active() {
                FindWindowResult::Handled
            } else {
                FindWindowResult::Inactive
            });
        }
        if self.input.is_none() {
            if self.message.take().is_some() {
                render_current_status(endpoint);
            }
            return Ok(FindWindowResult::Inactive);
        }
        let action = {
            let input = self.input.as_mut().expect("find input must exist");
            edit_prompt(key, input)
        };
        match action {
            PromptAction::Editing => {
                self.render()?;
                Ok(FindWindowResult::Handled)
            }
            PromptAction::Cancel => {
                self.input = None;
                render_current_status(endpoint);
                Ok(FindWindowResult::Handled)
            }
            PromptAction::Execute(query) => {
                self.input = None;
                self.execute(endpoint, query)
            }
        }
    }

    fn execute(
        &mut self,
        endpoint: &TmuxIpcEndpoint,
        query: String,
    ) -> io::Result<FindWindowResult> {
        let query = query.trim();
        if query.is_empty() {
            render_current_status(endpoint);
            return Ok(FindWindowResult::Handled);
        }
        let (active, indexes, names) = query_windows(endpoint)?;
        if let Some(index) = matching_window(
            active,
            indexes.as_slice(),
            names.as_slice(),
            query,
        ) {
            select_window(endpoint, index)?;
            render_current_status(endpoint);
            return Ok(if index == active {
                FindWindowResult::Handled
            } else {
                FindWindowResult::Selected(active)
            });
        }
        self.message = Some(
            terman_common::builtin_tmux_find_no_match_hint(query),
        );
        if let Some(message) = self.message.as_deref() {
            render_status_line(message)?;
        }
        Ok(FindWindowResult::Handled)
    }

    fn render(&self) -> io::Result<()> {
        let input = self.input.as_deref().unwrap_or_default();
        render_status_line(
            &terman_common::builtin_tmux_find_prompt_hint(input),
        )
    }
}

fn query_windows(
    endpoint: &TmuxIpcEndpoint,
) -> io::Result<(u32, Vec<u32>, Vec<String>)> {
    match request_endpoint_response(endpoint, TmuxIpcRequest::Info)? {
        TmuxIpcResponse::Info {
            active_window,
            window_indexes,
            window_names,
            ..
        } => Ok((active_window, window_indexes, window_names)),
        TmuxIpcResponse::Rejected { reason } => Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            reason,
        )),
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_tmux_unexpected_response_hint(
                &format!("{response:?}"),
            ),
        )),
    }
}

fn matching_window(
    active: u32,
    indexes: &[u32],
    names: &[String],
    query: &str,
) -> Option<u32> {
    if indexes.is_empty() {
        return None;
    }
    let query = query.to_lowercase();
    let active_position = indexes
        .iter()
        .position(|index| *index == active)
        .unwrap_or(indexes.len() - 1);
    (1..=indexes.len())
        .map(|offset| (active_position + offset) % indexes.len())
        .find(|position| {
            let index = indexes[*position];
            index.to_string() == query
                || names
                    .get(*position)
                    .is_some_and(|name| name.to_lowercase().contains(&query))
        })
        .and_then(|position| indexes.get(position).copied())
}

fn render_current_status(endpoint: &TmuxIpcEndpoint) {
    if let Ok(status) = query_status_line(endpoint) {
        let _ = render_status_line(&status);
    }
}

#[cfg(test)]
mod tests {
    use super::matching_window;

    #[test]
    fn wraps_after_the_active_window() {
        let indexes = [0, 1, 2];
        let names = ["editor", "logs", "build"].map(String::from);
        assert_eq!(
            matching_window(1, &indexes, &names, "EDIT"),
            Some(0)
        );
    }

    #[test]
    fn matches_a_window_index() {
        let indexes = [2, 7];
        let names = ["editor", "logs"].map(String::from);
        assert_eq!(
            matching_window(2, &indexes, &names, "7"),
            Some(7)
        );
    }
}