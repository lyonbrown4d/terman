use std::io;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    attach_command_exec::execute_attached_command,
    attach_keys::is_key_press,
    attach_status::{query_status_line, render_status_line},
    ipc::TmuxIpcEndpoint,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CommandPromptResult {
    Continue,
    Stop,
}

#[derive(Default)]
pub(crate) struct CommandPromptState {
    input: Option<String>,
    message: Option<String>,
}

impl CommandPromptState {
    pub(crate) fn begin(&mut self) -> io::Result<()> {
        self.message = None;
        self.input = Some(String::new());
        render_status_line(&terman_common::builtin_tmux_command_prompt_hint(""))
    }

    pub(crate) fn is_active(&self) -> bool {
        self.input.is_some()
    }

    pub(crate) fn status_override(&self) -> Option<String> {
        self.input
            .as_deref()
            .map(terman_common::builtin_tmux_command_prompt_hint)
            .or_else(|| self.message.clone())
    }

    pub(crate) fn handle_key(
        &mut self,
        endpoint: &TmuxIpcEndpoint,
        client_id: &str,
        key: &KeyEvent,
    ) -> io::Result<Option<CommandPromptResult>> {
        if !is_key_press(key) {
            return Ok(self.input.as_ref().map(|_| CommandPromptResult::Continue));
        }
        if self.input.is_none() {
            if self.message.take().is_some() {
                render_current_status(endpoint);
            }
            return Ok(None);
        }
        let action = {
            let input = self.input.as_mut().expect("command input must exist");
            edit_prompt(key, input)
        };
        match action {
            PromptAction::Editing => {
                self.render();
                Ok(Some(CommandPromptResult::Continue))
            }
            PromptAction::Cancel => {
                self.input = None;
                render_current_status(endpoint);
                Ok(Some(CommandPromptResult::Continue))
            }
            PromptAction::Execute(command) => {
                self.input = None;
                self.execute(endpoint, client_id, command)
            }
        }
    }

    fn execute(
        &mut self,
        endpoint: &TmuxIpcEndpoint,
        client_id: &str,
        command: String,
    ) -> io::Result<Option<CommandPromptResult>> {
        if command.trim().is_empty() {
            render_current_status(endpoint);
            return Ok(Some(CommandPromptResult::Continue));
        }
        match execute_attached_command(endpoint, client_id, command.as_str()) {
            Ok(stop) => {
                if !stop {
                    render_current_status(endpoint);
                }
                Ok(Some(if stop {
                    CommandPromptResult::Stop
                } else {
                    CommandPromptResult::Continue
                }))
            }
            Err(error) => {
                self.message = Some(error.to_string());
                if let Some(message) = self.message.as_deref() {
                    let _ = render_status_line(message);
                }
                Ok(Some(CommandPromptResult::Continue))
            }
        }
    }

    fn render(&self) {
        if let Some(input) = self.input.as_deref() {
            let status = terman_common::builtin_tmux_command_prompt_hint(input);
            let _ = render_status_line(&status);
        }
    }
}

enum PromptAction {
    Editing,
    Cancel,
    Execute(String),
}

fn edit_prompt(key: &KeyEvent, input: &mut String) -> PromptAction {
    match key.code {
        KeyCode::Enter => PromptAction::Execute(std::mem::take(input)),
        KeyCode::Esc => PromptAction::Cancel,
        KeyCode::Char('c' | 'C') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            PromptAction::Cancel
        }
        KeyCode::Backspace => {
            input.pop();
            PromptAction::Editing
        }
        KeyCode::Char('u' | 'U') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            input.clear();
            PromptAction::Editing
        }
        KeyCode::Char(ch)
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            input.push(ch);
            PromptAction::Editing
        }
        _ => PromptAction::Editing,
    }
}

fn render_current_status(endpoint: &TmuxIpcEndpoint) {
    if let Ok(status) = query_status_line(endpoint) {
        let _ = render_status_line(&status);
    }
}
