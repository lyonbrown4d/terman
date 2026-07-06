use std::{env, fs, io, path::PathBuf};

use super::{
    control_parse::control_command_payload,
    control_session::send_session_control_request,
};
use crate::{ScreenArgs, ipc::ScreenIpcRequest};

pub(super) fn request_chdir_command(
    args: &ScreenArgs,
    inline_payload: &str,
    extra_args: &[String],
) -> io::Result<()> {
    let payload = control_command_payload(inline_payload, extra_args);
    let path = if payload.trim().is_empty() {
        home_directory().ok_or_else(home_required_error)?
    } else {
        PathBuf::from(payload.trim())
    };
    let path = fs::canonicalize(path)?;
    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            terman_common::builtin_screen_control_chdir_directory_required_hint(),
        ));
    }
    send_session_control_request(args, ScreenIpcRequest::SetDefaultCwd { path })
}

fn home_directory() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn home_required_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_chdir_home_required_hint(),
    )
}

#[cfg(test)]
mod tests {
    use super::home_directory;

    #[test]
    fn resolves_home_directory_from_environment() {
        assert!(home_directory().is_some());
    }
}