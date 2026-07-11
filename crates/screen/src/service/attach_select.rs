use std::io;

use super::{

    control_select::resolve_window_selector,
    ipc_client::{request_endpoint_response, send_control_request},
};
use crate::{
    ipc::{ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse},
    terminal_prompt::read_screen_prompt,
};

pub(crate) fn prompt_screen_select(endpoint: &ScreenIpcEndpoint) -> io::Result<()> {
    let prompt = terman_common::builtin_screen_attach_select_prompt_hint();
    let Some(selector) = read_screen_prompt(prompt.as_str())? else {
        return Ok(());
    };
    if selector == "-" {
        return send_control_request(endpoint, ScreenIpcRequest::LastWindow);
    }

    match request_endpoint_response(endpoint, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            active_window,
            windows,
            ..
        } => {
            let index = match resolve_window_selector(&selector, active_window, &windows) {
                Ok(index) => index,
                Err(error) => {
                    println!("{error}");
                    return Ok(());
                }
            };
            send_control_request(endpoint, ScreenIpcRequest::SelectWindow { index })
        }
        ScreenIpcResponse::Rejected { reason } => {
            Err(io::Error::new(io::ErrorKind::Unsupported, reason))
        }
        response => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            terman_common::builtin_screen_control_unexpected_response_hint(&format!(
                "{response:?}"
            )),
        )),
    }
}