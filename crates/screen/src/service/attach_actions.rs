use std::io;


use super::{
    attach_buffer::{read_attach_buffer, remove_attach_buffer, write_attach_buffer},
    attach_number::print_attach_number,
    attach_output::{
        print_attach_displays, print_attach_hardcopy, print_attach_help, print_attach_info,
        print_attach_license, print_attach_time, print_attach_version, print_attach_windows,
    },
    attach_size::{fit_attach_window, toggle_attach_width},
    attach_termcap::print_attach_dumptermcap,
    ipc_client::send_control_request,
};
use crate::{
    ipc::{ScreenIpcEndpoint, ScreenIpcRequest},
    terminal_input::ScreenInputAction,
};

pub(super) enum AttachActionResult {
    CopyMode,
    CommandPrompt,
    SelectPrompt,
    TitlePrompt,
    WindowList,
    Continue,
    Stop,
}

pub(super) fn handle_attach_action(
    endpoint: &ScreenIpcEndpoint,
    client_id: &str,
    action: ScreenInputAction,
) -> io::Result<AttachActionResult> {
    match action {
        ScreenInputAction::CopyMode => return Ok(AttachActionResult::CopyMode),
        ScreenInputAction::CommandPrompt => return Ok(AttachActionResult::CommandPrompt),
        ScreenInputAction::Bytes(bytes) => {
            send_control_request(endpoint, ScreenIpcRequest::Input { bytes })?;
        }
        ScreenInputAction::Clear => send_control_request(endpoint, ScreenIpcRequest::Clear)?,
        ScreenInputAction::Detach => {
            send_control_request(
                endpoint,
                ScreenIpcRequest::DetachClient {
                    client_id: client_id.to_string(),
                },
            )?;
            return Ok(AttachActionResult::Stop);
        }
        ScreenInputAction::DetachAll => {
            send_control_request(endpoint, ScreenIpcRequest::DetachAll)?;
            return Ok(AttachActionResult::Stop);
        }
        ScreenInputAction::Displays => print_attach_displays(endpoint)?,
        ScreenInputAction::DumpTermcap => print_attach_dumptermcap(endpoint)?,
        ScreenInputAction::Fit => fit_attach_window(endpoint)?,
        ScreenInputAction::Hardcopy => print_attach_hardcopy(endpoint)?,
        ScreenInputAction::Help => print_attach_help()?,
        ScreenInputAction::Info => print_attach_info(endpoint)?,
        ScreenInputAction::Kill => send_control_request(endpoint, ScreenIpcRequest::KillWindow)?,
        ScreenInputAction::LastMessage => {
            send_control_request(endpoint, ScreenIpcRequest::LastMessage)?;
        }
        ScreenInputAction::LastWindow => {
            send_control_request(endpoint, ScreenIpcRequest::LastWindow)?;
        }
        ScreenInputAction::License => print_attach_license()?,
        ScreenInputAction::LogToggle => send_control_request(endpoint, ScreenIpcRequest::ToggleLog)?,
        ScreenInputAction::MonitorToggle => {
            send_control_request(endpoint, ScreenIpcRequest::SetMonitor { enabled: None })?;
        }
        ScreenInputAction::SilenceToggle => {
            send_control_request(endpoint, ScreenIpcRequest::ToggleSilence)?;
        }
        ScreenInputAction::NewWindow => {
            send_control_request(endpoint, ScreenIpcRequest::NewWindow { command: None })?;
        }
        ScreenInputAction::NextWindow => {
            send_control_request(endpoint, ScreenIpcRequest::NextWindow)?;
        }
        ScreenInputAction::Number => print_attach_number(endpoint)?,
        ScreenInputAction::Paste => send_control_request(endpoint, ScreenIpcRequest::PasteBuffer)?,
        ScreenInputAction::PreviousWindow => {
            send_control_request(endpoint, ScreenIpcRequest::PreviousWindow)?;
        }
        ScreenInputAction::SplitRegion(axis) => {
            send_control_request(endpoint, ScreenIpcRequest::SplitRegion { axis })?;
        }
        ScreenInputAction::FocusRegion(target) => {
            send_control_request(endpoint, ScreenIpcRequest::FocusRegion { target })?;
        }
        ScreenInputAction::RemoveRegion => {
            send_control_request(endpoint, ScreenIpcRequest::RemoveRegion)?;
        }
        ScreenInputAction::OnlyRegion => {
            send_control_request(endpoint, ScreenIpcRequest::OnlyRegion)?;
        }
        ScreenInputAction::Quit => {
            send_control_request(endpoint, ScreenIpcRequest::Quit)?;
            return Ok(AttachActionResult::Stop);
        }
        ScreenInputAction::ReadBuffer => read_attach_buffer(endpoint)?,
        ScreenInputAction::RemoveBuffer => remove_attach_buffer(endpoint)?,
        ScreenInputAction::Reset => send_control_request(endpoint, ScreenIpcRequest::Reset)?,
        ScreenInputAction::Redisplay => {
            send_control_request(endpoint, ScreenIpcRequest::Redisplay)?;
        }
        ScreenInputAction::WrapToggle => send_control_request(endpoint, ScreenIpcRequest::SetWrap { enabled: None })?,
        ScreenInputAction::SelectPrompt => return Ok(AttachActionResult::SelectPrompt),
        ScreenInputAction::WindowList => return Ok(AttachActionResult::WindowList),
        ScreenInputAction::SelectWindow(index) => {
            send_control_request(endpoint, ScreenIpcRequest::SelectWindow { index })?;
        }
        ScreenInputAction::Time => print_attach_time()?,
        ScreenInputAction::Title => return Ok(AttachActionResult::TitlePrompt),
        ScreenInputAction::Version => print_attach_version()?,
        ScreenInputAction::WidthToggle => toggle_attach_width(endpoint)?,
        ScreenInputAction::Windows => print_attach_windows(endpoint)?,
        ScreenInputAction::WriteBuffer => write_attach_buffer(endpoint)?,
    }
    Ok(AttachActionResult::Continue)
}

pub(super) fn sync_attach_terminal_size(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    let (cols, rows) = terman_common::current_terminal_size()?;
    send_control_request(endpoint, ScreenIpcRequest::Resize { cols, rows })
}
