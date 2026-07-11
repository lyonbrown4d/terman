use std::io;

use super::ipc_client::send_control_request;
use crate::{
    ipc::{ScreenIpcEndpoint, ScreenIpcRequest},
    terminal_prompt::read_screen_prompt,
};

pub(crate) fn prompt_screen_title(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    let prompt = terman_common::builtin_screen_attach_title_prompt_hint();
    let Some(title) = read_screen_prompt(prompt.as_str())? else {
        return Ok(());
    };
    send_control_request(endpoint, ScreenIpcRequest::SetWindowTitle { title })
}