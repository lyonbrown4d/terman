use std::{env, fs, io, path::PathBuf};

use directories::ProjectDirs;

use super::control_session::request_session_response;
use crate::{
    ScreenArgs,
    ipc::{ScreenIpcRequest, ScreenIpcResponse},
};

pub(super) fn request_dumptermcap_command(args: &ScreenArgs) -> io::Result<()> {
    let (session_name, cols, rows) = session_termcap_size(args)?;
    let path = screen_termcap_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let entry = termcap_entry(&session_name, cols, rows);
    fs::write(&path, entry)?;
    println!(
        "{}",
        terman_common::builtin_screen_control_dumptermcap_complete_hint(
            &path.display().to_string(),
        )
    );
    Ok(())
}

fn session_termcap_size(args: &ScreenArgs) -> io::Result<(String, u16, u16)> {
    match request_session_response(args, ScreenIpcRequest::Info)? {
        ScreenIpcResponse::Info {
            session_name,
            cols,
            rows,
            ..
        } => Ok((session_name, cols.unwrap_or(80), rows.unwrap_or(24))),
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

fn screen_termcap_path() -> PathBuf {
    ProjectDirs::from("", "", "terman")
        .map(|dirs| dirs.data_local_dir().join("screen").join(".termcap"))
        .unwrap_or_else(|| env::temp_dir().join("terman-screen").join(".termcap"))
}

fn termcap_entry(session_name: &str, cols: u16, rows: u16) -> String {
    let name = termcap_name(session_name);
    format!(
        "terman-screen-{name}|terman screen session {session_name}:\\\n\
         \t:co#{cols}:li#{rows}:am:bs:mi:ms:\\\n\
         \t:cl=\\E[H\\E[2J:cm=\\E[%i%d;%dH:ce=\\E[K:cd=\\E[J:\\\n\
         \t:so=\\E[7m:se=\\E[27m:us=\\E[4m:ue=\\E[24m:\\\n\
         \t:md=\\E[1m:me=\\E[0m:nd=\\E[C:up=\\E[A:\n"
    )
}

fn termcap_name(session_name: &str) -> String {
    let name: String = session_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect();
    if name.is_empty() {
        String::from("session")
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::{termcap_entry, termcap_name};

    #[test]
    fn sanitizes_termcap_names() {
        assert_eq!(termcap_name("dev/session"), "dev-session");
        assert_eq!(termcap_name(""), "session");
    }

    #[test]
    fn builds_termcap_entry_with_size() {
        let entry = termcap_entry("dev", 132, 40);

        assert!(entry.contains("terman-screen-dev"));
        assert!(entry.contains(":co#132:li#40:"));
    }
}