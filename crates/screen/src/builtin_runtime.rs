use std::{io, sync::mpsc, time::Duration};

use crossterm::{
    event::{self, Event},
    terminal,
};

use crate::{
    ScreenArgs,
    blanker::ScreenBlanker,
    confirmation::{ScreenConfirmation, prompt_screen_confirmation},
    builtin_input::handle_builtin_input_action,
    builtin_mouse::{
        ScreenMouseState, disable_mouse_capture, enable_mouse_capture, handle_builtin_mouse, handle_builtin_window_list_key, open_builtin_window_list,
    },
    builtin_output::{publish_window_redraw, write_region_frame},
    copy_mode::{ScreenCopyMode, ScreenCopyResult},
    ipc::ScreenIpcEndpoint,
    session_core::{ScreenControlEvent, ScreenSessionBus},
    terminal_input::{ScreenInputAction, ScreenInputDecoder},
    window_runtime::{ScreenWindowRuntime, resize_windows},
};

pub(crate) struct RawMode;

impl RawMode {
    pub(crate) fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        enable_mouse_capture()?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        disable_mouse_capture();
        let _ = terminal::disable_raw_mode();
    }
}

pub(crate) fn screen_session_endpoint(args: &ScreenArgs) -> ScreenIpcEndpoint {
    args.session_name
        .as_deref()
        .map(ScreenIpcEndpoint::for_new_session)
        .unwrap_or_else(|| ScreenIpcEndpoint::for_session("anonymous"))
}

pub(crate) fn resolve_size(cols_override: Option<u16>, rows_override: Option<u16>) -> (u16, u16) {
    let (cols, rows) = terman_common::current_terminal_size().unwrap_or((120, 32));
    (cols_override.unwrap_or(cols), rows_override.unwrap_or(rows))
}

pub(crate) fn poll_terminal_event(
    session_bus: &ScreenSessionBus,
    control_tx: &mpsc::Sender<ScreenControlEvent>,
    input_decoder: &mut ScreenInputDecoder,
    copy_mode: &mut Option<ScreenCopyMode>,
    blanker: &mut ScreenBlanker,
    windows: &mut [ScreenWindowRuntime],
    active_window: &mut usize,
    mouse_state: &mut ScreenMouseState,
) -> io::Result<()> {
    if !event::poll(Duration::from_millis(16))? {
        return Ok(());
    }
    match event::read()? {
        Event::Mouse(mouse) => {
            if blanker.is_active() {
                if blanker.dismiss_mouse(&mouse)? {
                    restore_builtin_display(session_bus);
                }
                return Ok(());
            }
            if let Some(mode) = copy_mode.as_mut() {
                if mode.handle_mouse(mouse) {
                    mode.render()?;
                }
            } else {
                handle_builtin_mouse(session_bus, windows, active_window, mouse_state, mouse);
            }
        }
        Event::Key(key) => {
            if blanker.is_active() {
                if blanker.dismiss_key(&key)? {
                    restore_builtin_display(session_bus);
                }
                return Ok(());
            }
            if let Some(mode) = copy_mode.as_mut() {
                match mode.handle_key(key) {
                    ScreenCopyResult::Continue => mode.render()?,
                    ScreenCopyResult::Cancel => {
                        *copy_mode = None;
                        restore_builtin_display(session_bus);
                    }
                    ScreenCopyResult::Copy(bytes) => {
                        session_bus.set_paste_buffer(bytes);
                        *copy_mode = None;
                        restore_builtin_display(session_bus);
                    }
                }
            } else if handle_builtin_window_list_key(
                session_bus,
                windows,
                active_window,
                mouse_state,
                &key,
            ) {
            } else if let Some(action) = input_decoder.decode_key(key) {
                match action {
                    ScreenInputAction::Blank => blanker.activate()?,
                    ScreenInputAction::CopyMode => {
                        let (cols, rows) = terman_common::current_terminal_size()?;
                        let mode = ScreenCopyMode::from_replay(
                            &session_bus.hardcopy_snapshot(true),
                            cols,
                            rows,
                        );
                        mode.render()?;
                        *copy_mode = Some(mode);
                    }
                    ScreenInputAction::WindowList => {
                        open_builtin_window_list(session_bus, mouse_state);
                    }
                    ScreenInputAction::Kill => {
                        if prompt_screen_confirmation(
                            ScreenConfirmation::KillWindow,
                        )? {
                            handle_builtin_input_action(
                                session_bus,
                                control_tx,
                                ScreenInputAction::Kill,
                            )?;
                        }
                        restore_builtin_display(session_bus);
                    }
                    ScreenInputAction::Quit => {
                        if prompt_screen_confirmation(
                            ScreenConfirmation::QuitSession,
                        )? {
                            handle_builtin_input_action(
                                session_bus,
                                control_tx,
                                ScreenInputAction::Quit,
                            )?;
                        }
                        restore_builtin_display(session_bus);
                    }
                    action => {
                        handle_builtin_input_action(
                            session_bus,
                            control_tx,
                            action,
                        )?;
                    }
                }
            }
        }
        Event::Resize(cols, rows) => {
            resize_windows(windows, cols, rows);
            session_bus.publish_resize(cols, rows);
            if blanker.is_active() {
                blanker.render()?;
            } else if let Some(mode) = copy_mode.as_mut() {
                mode.resize(cols, rows);
                mode.render()?;
            } else if mouse_state.list_open() {
                open_builtin_window_list(session_bus, mouse_state);
            } else if let Some(frame) = session_bus.publish_region_redraw() {
                write_region_frame(&frame);
            }
        }
        _ => {}
    }
    Ok(())
}

fn restore_builtin_display(session_bus: &ScreenSessionBus) {
    if let Some(frame) = session_bus.publish_region_redraw() {
        write_region_frame(&frame);
    } else {
        publish_window_redraw(session_bus, &session_bus.hardcopy_snapshot(false), true);
    }
}