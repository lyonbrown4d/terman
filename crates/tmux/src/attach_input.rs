use std::io;

use crossterm::event::{KeyCode, KeyEvent};

use crate::{
    attach_keys::{
        TmuxPrefixCommand, is_detach_key, is_key_press, is_tmux_prefix_key, key_event_bytes,
        tmux_prefix_bytes, tmux_prefix_command,
    },
    attach_pane::{
        kill_current_pane, select_next_pane, split_current_pane, toggle_current_pane_zoom,
    },
    attach_rename::handle_rename_input,
    attach_status::{
        KILL_PANE_CONFIRM_STATUS, KILL_WINDOW_CONFIRM_STATUS, query_status_line,
        render_status_line,
    },
    attach_window::{
        current_active_window, handle_window_command, kill_current_window, select_window,
    },
    attach_window_list::render_window_list_status,
    ipc::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse},
    service::request_endpoint_response,
};

#[derive(Clone, Copy)]
enum KillTarget {
    Pane,
    Window,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AttachInputResult {
    Continue,
    Stop,
    EnterCopyMode,
}

#[derive(Default)]
pub(crate) struct AttachInputMode {
    prefix_pending: bool,
    kill_pending: Option<KillTarget>,
    rename_input: Option<String>,
    last_window: Option<u32>,
}

impl AttachInputMode {
    pub(crate) fn handle_key(
        &mut self,
        endpoint: &TmuxIpcEndpoint,
        client_id: &str,
        key: KeyEvent,
    ) -> io::Result<AttachInputResult> {
        if !is_key_press(&key) {
            return Ok(AttachInputResult::Continue);
        }
        if self.handle_rename(endpoint, &key)? {
            return Ok(AttachInputResult::Continue);
        }
        if let Some(target) = self.kill_pending {
            if !handle_kill_confirmation(endpoint, &key, target)? {
                self.kill_pending = None;
            }
            return Ok(AttachInputResult::Continue);
        }
        if self.prefix_pending {
            return self.handle_prefix(endpoint, client_id, &key);
        }
        if is_tmux_prefix_key(&key) {
            self.prefix_pending = true;
            let _ = render_status_line(&terman_common::builtin_tmux_prefix_status_hint());
            return Ok(AttachInputResult::Continue);
        }
        if let Some(bytes) = key_event_bytes(&key) {
            send_input(endpoint, bytes)?;
        }
        Ok(AttachInputResult::Continue)
    }

    fn handle_rename(&mut self, endpoint: &TmuxIpcEndpoint, key: &KeyEvent) -> io::Result<bool> {
        let Some(input) = self.rename_input.as_mut() else {
            return Ok(false);
        };
        handle_rename_input(endpoint, key, input)?;
        if matches!(key.code, KeyCode::Enter | KeyCode::Esc) {
            self.rename_input = None;
        }
        Ok(true)
    }

    fn handle_prefix(
        &mut self,
        endpoint: &TmuxIpcEndpoint,
        client_id: &str,
        key: &KeyEvent,
    ) -> io::Result<AttachInputResult> {
        self.prefix_pending = false;
        if is_detach_key(key) {
            send_request(
                endpoint,
                TmuxIpcRequest::DetachClient {
                    client_id: client_id.to_string(),
                },
            )?;
            return Ok(AttachInputResult::Stop);
        }
        match tmux_prefix_command(key) {
            Some(TmuxPrefixCommand::CopyMode) => Ok(AttachInputResult::EnterCopyMode),
            Some(TmuxPrefixCommand::PasteBuffer) => {
                send_request(endpoint, TmuxIpcRequest::PasteBuffer)?;
                let _ = render_current_status(endpoint);
                Ok(AttachInputResult::Continue)
            }
            Some(command) => {
                self.handle_prefix_command(endpoint, command)?;
                Ok(AttachInputResult::Continue)
            }
            None => {
                send_input(endpoint, tmux_prefix_bytes())?;
                let _ = render_current_status(endpoint);
                Ok(AttachInputResult::Continue)
            }
        }
    }

    fn handle_prefix_command(
        &mut self,
        endpoint: &TmuxIpcEndpoint,
        command: TmuxPrefixCommand,
    ) -> io::Result<()> {
        match command {
            TmuxPrefixCommand::KillPane => {
                self.kill_pending = Some(KillTarget::Pane);
                let _ = render_status_line(KILL_PANE_CONFIRM_STATUS);
            }
            TmuxPrefixCommand::KillWindow => {
                self.kill_pending = Some(KillTarget::Window);
                let _ = render_status_line(KILL_WINDOW_CONFIRM_STATUS);
            }
            TmuxPrefixCommand::SplitHorizontal => {
                split_current_pane(endpoint, true)?;
                let _ = render_current_status(endpoint);
            }
            TmuxPrefixCommand::SplitVertical => {
                split_current_pane(endpoint, false)?;
                let _ = render_current_status(endpoint);
            }
            TmuxPrefixCommand::TogglePaneZoom => {
                toggle_current_pane_zoom(endpoint)?;
                let _ = render_current_status(endpoint);
            }
            TmuxPrefixCommand::NextPane => {
                select_next_pane(endpoint)?;
                let _ = render_current_status(endpoint);
            }
            TmuxPrefixCommand::RenameWindow => {
                self.rename_input = Some(String::new());
                let _ = render_status_line("tmux rename | ");
            }
            TmuxPrefixCommand::ListWindows => {
                let _ = render_window_list_status(endpoint)?;
            }
            TmuxPrefixCommand::Help => {
                let _ = render_status_line(&terman_common::builtin_tmux_attach_help());
            }
            TmuxPrefixCommand::LastWindow => self.select_last_window(endpoint)?,
            TmuxPrefixCommand::CopyMode | TmuxPrefixCommand::PasteBuffer => {
                unreachable!("copy and paste are handled before command dispatch")
            }
            command => {
                self.track_last_window(endpoint, command)?;
                handle_window_command(endpoint, command)?;
                let _ = render_current_status(endpoint);
            }
        }
        Ok(())
    }

    fn select_last_window(&mut self, endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
        let Some(index) = self.last_window else {
            return render_current_status(endpoint);
        };
        let active_window = current_active_window(endpoint)?;
        select_window(endpoint, index)?;
        self.last_window = Some(active_window);
        render_current_status(endpoint)
    }

    fn track_last_window(
        &mut self,
        endpoint: &TmuxIpcEndpoint,
        command: TmuxPrefixCommand,
    ) -> io::Result<()> {
        if active_changing_command(command) {
            self.last_window = Some(current_active_window(endpoint)?);
        }
        Ok(())
    }
}

fn active_changing_command(command: TmuxPrefixCommand) -> bool {
    matches!(
        command,
        TmuxPrefixCommand::CreateWindow
            | TmuxPrefixCommand::NextWindow
            | TmuxPrefixCommand::PreviousWindow
            | TmuxPrefixCommand::SelectWindow(_)
    )
}

fn handle_kill_confirmation(
    endpoint: &TmuxIpcEndpoint,
    key: &KeyEvent,
    target: KillTarget,
) -> io::Result<bool> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            match target {
                KillTarget::Pane => kill_current_pane(endpoint)?,
                KillTarget::Window => kill_current_window(endpoint)?,
            }
            let _ = render_current_status(endpoint);
            Ok(false)
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            let _ = render_current_status(endpoint);
            Ok(false)
        }
        _ => Ok(true),
    }
}

fn render_current_status(endpoint: &TmuxIpcEndpoint) -> io::Result<()> {
    render_status_line(&query_status_line(endpoint)?)
}

fn send_input(endpoint: &TmuxIpcEndpoint, bytes: Vec<u8>) -> io::Result<()> {
    send_request(endpoint, TmuxIpcRequest::Input { bytes })
}

fn send_request(endpoint: &TmuxIpcEndpoint, request: TmuxIpcRequest) -> io::Result<()> {
    match request_endpoint_response(endpoint, request)? {
        TmuxIpcResponse::Accepted => Ok(()),
        TmuxIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, reason))
        }
        _ => Ok(()),
    }
}