use std::{io, sync::mpsc};

use crate::{
    builtin_buffer::{read_buffer_file, remove_buffer_file, write_buffer_file},
    session_core::{ScreenControlEvent, ScreenSessionBus},
    terminal_input::ScreenInputAction,
};

pub(crate) fn handle_builtin_input_action(
    bus: &ScreenSessionBus,
    control_tx: &mpsc::Sender<ScreenControlEvent>,
    action: ScreenInputAction,
) -> io::Result<()> {
    let event = match action {
        ScreenInputAction::Blank | ScreenInputAction::CopyMode | ScreenInputAction::WindowList =>
            unreachable!("interactive mode is handled by the runtime"),
        ScreenInputAction::BlankRegion => Some(ScreenControlEvent::BlankRegion),
        ScreenInputAction::Bytes(bytes) => Some(ScreenControlEvent::Input(bytes)),
        ScreenInputAction::NewWindow => Some(ScreenControlEvent::NewWindow { command: None }),
        ScreenInputAction::SelectWindow(index) => {
            Some(ScreenControlEvent::SelectWindow { index })
        }
        ScreenInputAction::NextWindow => Some(ScreenControlEvent::NextWindow),
        ScreenInputAction::PreviousWindow => Some(ScreenControlEvent::PreviousWindow),
        ScreenInputAction::LastWindow => Some(ScreenControlEvent::LastWindow),
        ScreenInputAction::Kill => Some(ScreenControlEvent::KillWindow),
        ScreenInputAction::Quit => Some(ScreenControlEvent::Terminate),
        ScreenInputAction::SplitRegion(axis) => {
            Some(ScreenControlEvent::SplitRegion { axis })
        }
        ScreenInputAction::FocusRegion(target) => {
            Some(ScreenControlEvent::FocusRegion { target })
        }
        ScreenInputAction::RemoveRegion => Some(ScreenControlEvent::RemoveRegion),
        ScreenInputAction::OnlyRegion => Some(ScreenControlEvent::OnlyRegion),
        ScreenInputAction::Paste => {
            Some(ScreenControlEvent::Input(bus.paste_buffer_snapshot()))
        }
        ScreenInputAction::ReadBuffer => {
            read_buffer_file(bus);
            None
        }
        ScreenInputAction::WriteBuffer => {
            write_buffer_file(bus);
            None
        }
        ScreenInputAction::RemoveBuffer => {
            remove_buffer_file(bus);
            None
        }
        ScreenInputAction::Clear => {
            bus.publish_display_control(b"[2J[H");
            None
        }
        ScreenInputAction::Reset => {
            bus.publish_display_control(b"c");
            None
        }
        ScreenInputAction::Redisplay => {
            bus.publish_transient_output(&bus.hardcopy_snapshot(false));
            None
        }
        ScreenInputAction::LogToggle => {
            let _ = bus.toggle_log_enabled();
            None
        }
        ScreenInputAction::MonitorToggle => {
            bus.set_monitor_enabled(None);
            None
        }
        ScreenInputAction::SilenceToggle => {
            bus.toggle_silence();
            None
        }
        ScreenInputAction::Detach | ScreenInputAction::DetachAll => {
            bus.publish_detach();
            None
        }
        ScreenInputAction::WrapToggle => {
            bus.set_wrap_enabled(None);
            None
        }
        ScreenInputAction::Fit => {
            let (cols, rows) = terman_common::current_terminal_size()?;
            Some(ScreenControlEvent::Resize { cols, rows })
        }
        ScreenInputAction::CommandPrompt
        | ScreenInputAction::SelectPrompt
        | ScreenInputAction::Displays
        | ScreenInputAction::DumpTermcap
        | ScreenInputAction::Hardcopy
        | ScreenInputAction::Help
        | ScreenInputAction::Info
        | ScreenInputAction::LastMessage
        | ScreenInputAction::License
        | ScreenInputAction::Number
        | ScreenInputAction::Time
        | ScreenInputAction::Title
        | ScreenInputAction::Version
        | ScreenInputAction::WidthToggle
        | ScreenInputAction::Windows => None,
    };

    if let Some(event) = event {
        control_tx
            .send(event)
            .map_err(|err| io::Error::new(io::ErrorKind::BrokenPipe, err.to_string()))?;
    }
    Ok(())
}
