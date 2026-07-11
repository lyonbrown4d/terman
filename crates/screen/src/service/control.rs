use std::io;

use super::{
    control_at::request_at_command,
    control_buffer::{
        request_bufferfile_command, request_readbuf_command, request_removebuf_command,
        request_writebuf_command,
    },
    control_chdir::request_chdir_command,
    control_colon::request_colon_command,
    control_displays::request_displays_command,
    control_env::{request_setenv_command, request_unsetenv_command},
    control_hardcopy::{
        request_hardcopy_append_command, request_hardcopy_command, request_hardcopydir_command,
    },
    control_info::{request_dinfo_command, request_info_command},
    control_local::request_local_control_command,
    control_log::{request_deflog_command, request_log_command, request_logtstamp_command},
    control_monitor::request_monitor_command,
    control_silence::request_silence_command,
    control_number::request_number_command,
    control_register::{request_process_command, request_readreg_command, request_register_command},
    control_scrollback::{request_defscrollback_command, request_scrollback_command},
    control_select::request_select_command,
    control_session::{
        request_echo_command, request_kill_command, request_logfile_command, request_new_window_command,
        request_paste_command, request_pastefile_command, request_resize_command, request_session_response, request_stuff_command, request_title_command, 
        send_session_control_request, send_targeted_session_control_request,
    },
    control_shell::{request_shell_command, request_shelltitle_command, request_term_command},
    control_size::{request_fit_command, request_size_command},
    control_source::request_source_command,
    control_termcap::request_dumptermcap_command,
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
        "quit" => send_session_control_request(args, ScreenIpcRequest::Quit),
        "kill" => request_kill_command(args),
        "detach" | "pow_detach" => send_session_control_request(args, ScreenIpcRequest::DetachAll),
        "bell" => send_session_control_request(args, ScreenIpcRequest::Bell),
        "clear" => send_session_control_request(args, ScreenIpcRequest::Clear),
        "reset" => send_session_control_request(args, ScreenIpcRequest::Reset),
        "redisplay" => send_session_control_request(args, ScreenIpcRequest::Redisplay),
        "echo" | "wall" => request_echo_command(args, inline_payload, extra_args),
        "eval" => request_eval_command(args, inline_payload, extra_args),
        "at" => request_at_command(args, inline_payload, extra_args, execute_control_command),
        "colon" => request_colon_command(args, inline_payload, extra_args, execute_control_command),
        "source" => request_source_command(args, inline_payload, extra_args, execute_control_command),
        "screen" => request_new_window_command(args, inline_payload, extra_args),
        "shell" | "defshell" => request_shell_command(args, inline_payload, extra_args),
        "shelltitle" => request_shelltitle_command(args, inline_payload, extra_args),
        "term" => request_term_command(args, inline_payload, extra_args),
        "chdir" => request_chdir_command(args, inline_payload, extra_args),
        "setenv" => request_setenv_command(args, inline_payload, extra_args),
        "unsetenv" => request_unsetenv_command(args, inline_payload, extra_args),
        "displays" => request_displays_command(args, request_session_response),
        "windows" | "windowlist" => request_windows_command(args, request_session_response),
        "info" => request_info_command(args, request_session_response),
        "dinfo" => request_dinfo_command(args, request_session_response),
        "dumptermcap" => request_dumptermcap_command(args),
        "lastmsg" => send_session_control_request(args, ScreenIpcRequest::LastMessage),
        "monitor" => request_monitor_command(args, inline_payload, extra_args),
        "silence" => request_silence_command(args, inline_payload, extra_args),
        "hardcopy" => request_hardcopy_command(args, inline_payload, extra_args),
        "hardcopydir" => request_hardcopydir_command(args, inline_payload, extra_args),
        "hardcopy_append" => request_hardcopy_append_command(args, inline_payload, extra_args),
        "log" => request_log_command(args, inline_payload, extra_args),
        "deflog" => request_deflog_command(args, inline_payload, extra_args),
        "logtstamp" => request_logtstamp_command(args, inline_payload, extra_args),
        "logfile" => request_logfile_command(args, inline_payload, extra_args),
        "paste" => request_paste_command(args, inline_payload, extra_args),
        "pastefile" => request_pastefile_command(args, inline_payload, extra_args),
        "bufferfile" => request_bufferfile_command(args, inline_payload, extra_args),
        "process" => request_process_command(args, inline_payload, extra_args),
        "register" => request_register_command(args, inline_payload, extra_args),
        "readreg" => request_readreg_command(args, inline_payload, extra_args),
        "readbuf" => request_readbuf_command(args, inline_payload, extra_args),
        "removebuf" => request_removebuf_command(args),
        "writebuf" => request_writebuf_command(args, inline_payload, extra_args),
        "resize" => request_resize_command(args, inline_payload, extra_args),
        "fit" => request_fit_command(args),
        "width" | "height" => request_size_command(args, &command, inline_payload, extra_args),
        "select" => request_select_command(args, inline_payload, extra_args, request_session_response),
        "number" => request_number_command(args, inline_payload, extra_args),
        "scrollback" => request_scrollback_command(args, inline_payload, extra_args),
        "defscrollback" => request_defscrollback_command(args, inline_payload, extra_args),
        "next" | "prev" | "previous" | "other" | "split" | "focus" | "remove" | "only" => {
            request_window_navigation_command(args, &command, request_session_response)
        },
        "sessionname" => request_sessionname_command(args, inline_payload, extra_args),
        "title" | "aka" => request_title_command(args, inline_payload, extra_args),
        "stuff" => request_stuff_command(args, inline_payload, extra_args),
        "meta" => send_targeted_session_control_request(args, ScreenIpcRequest::Input { bytes: vec![0x01] }),
        "xon" => send_targeted_session_control_request(args, ScreenIpcRequest::Input { bytes: vec![0x11] }),
        "xoff" => send_targeted_session_control_request(args, ScreenIpcRequest::Input { bytes: vec![0x13] }),
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