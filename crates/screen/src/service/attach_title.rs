use std::io;

use super::{
    attach_prompt::read_attach_prompt,
    ipc_client::send_control_request,
};
use crate::ipc::{ScreenIpcEndpoint, ScreenIpcRequest};

pub(super) fn prompt_attach_title(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    let prompt = terman_common::builtin_screen_attach_title_prompt_hint();
    let Some(title) = read_attach_prompt(prompt.as_str())? else {
        return Ok(());
    };
    send_control_request(endpoint, ScreenIpcRequest::SetWindowTitle { title })
}