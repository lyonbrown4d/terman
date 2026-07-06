use std::io;

use super::{
    control_at::request_at_command,
    control_colon::request_colon_command,
    control_displays::request_displays_command,
    control_info::request_info_command,
    control_local::request_local_control_command,
    control_select::request_select_command,
    control_session::{
        request_echo_command, request_hardcopy_command, request_log_command, request_logfile_command,
        request_paste_command, request_pastefile_command, request_readbuf_command, request_resize_command, request_scrollback_command, request_session_response, request_stuff_command, request_title_command,
        send_session_control_request,
    },
    control_source::request_source_command,
    control_window_nav::request_window_navigation_command,
    control_windows::request_windows_command,
    sessionname::request_sessionname_command,
};
use crate::{ScreenArgs, ipc::ScreenIpcRequest};

pub(crate) fn request_screen_control_command(args: &ScreenArgs) -> io::Result<()> {
    let Some(command_text) = args
        .execute
        .as_deref()
        .map(str::trim)
        .filter(|command| !command.is_empty())
    else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_command_required_hint(),
        ));
    };

    let (command, inline_payload) = split_control_command(command_text);
    execute_control_command(args, command, inline_payload, &args.execute_args)
}

fn execute_control_command(
    args: &ScreenArgs,
    command: &str,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let command = command.to_ascii_lowercase();
    if let Some(result) = request_local_control_command(&command, inline_payload, extra_args) {
        return result;
    }

    match command.as_str() {
        "quit" | "kill" => send_session_control_request(args, ScreenIpcRequest::Quit),
        "detach" | "pow_detach" => send_session_control_request(args, ScreenIpcRequest::DetachAll),
        "bell" => send_session_control_request(args, ScreenIpcRequest::Bell),
        "clear" => send_session_control_request(args, ScreenIpcRequest::Clear),
        "reset" => send_session_control_request(args, ScreenIpcRequest::Reset),
        "echo" | "wall" => request_echo_command(args, inline_payload, extra_args),
        "eval" => request_eval_command(args, inline_payload, extra_args),
        "at" => request_at_command(args, inline_payload, extra_args, execute_control_command),
        "colon" => request_colon_command(args, inline_payload, extra_args, execute_control_command),
        "source" => request_source_command(args, inline_payload, extra_args, execute_control_command),
        "displays" => request_displays_command(args, request_session_response),
        "windows" => request_windows_command(args, request_session_response),
        "info" => request_info_command(args, request_session_response),
        "hardcopy" => request_hardcopy_command(args, inline_payload, extra_args),
        "log" => request_log_command(args, inline_payload, extra_args),
        "logfile" => request_logfile_command(args, inline_payload, extra_args),
        "paste" => request_paste_command(args, inline_payload, extra_args),
        "pastefile" => request_pastefile_command(args, inline_payload, extra_args),
        "readbuf" => request_readbuf_command(args, inline_payload, extra_args),
        "resize" => request_resize_command(args, inline_payload, extra_args),
        "select" => request_select_command(args, inline_payload, extra_args, request_session_response),
        "scrollback" | "defscrollback" => {
            request_scrollback_command(args, inline_payload, extra_args)
        }
        "next" | "prev" => request_window_navigation_command(args, request_session_response),
        "sessionname" => request_sessionname_command(args, inline_payload, extra_args),
        "title" | "aka" => request_title_command(args, inline_payload, extra_args),
        "stuff" => request_stuff_command(args, inline_payload, extra_args),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_command_unsupported_hint(&command),
        )),
    }
}

fn request_eval_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let commands = eval_command_payloads(inline_payload, extra_args);
    if commands.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_command_required_hint(),
        ));
    }

    for command_text in commands {
        let (command, inline_payload) = split_control_command(&command_text);
        execute_control_command(args, command, inline_payload, &[])?;
    }
    Ok(())
}

fn split_control_command(command: &str) -> (&str, &str) {
    let command = command.trim();
    let command_end = command.find(char::is_whitespace).unwrap_or(command.len());
    let verb = &command[..command_end];
    let payload = command[command_end..].trim_start();
    (verb, payload)
}

fn eval_command_payloads(inline_payload: &str, extra_args: &[String]) -> Vec<String> {
    let mut commands = Vec::new();
    if !inline_payload.trim().is_empty() {
        commands.push(inline_payload.trim().to_string());
    }
    commands.extend(
        extra_args
            .iter()
            .map(|arg| arg.trim())
            .filter(|arg| !arg.is_empty())
            .map(ToString::to_string),
    );
    commands
}

#[cfg(test)]
mod tests {
    use super::eval_command_payloads;

    #[test]
    fn builds_eval_command_payloads() {
        assert_eq!(
            eval_command_payloads("stuff echo hi", &["info".into()]),
            vec![String::from("stuff echo hi"), String::from("info")]
        );
        assert!(eval_command_payloads("  ", &[]).is_empty());
    }
}