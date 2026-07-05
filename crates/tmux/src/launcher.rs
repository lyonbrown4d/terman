use std::{
    env, io,
    process::{Command, Stdio},
};

pub(crate) fn spawn_detached_tmux_server(
    session_name: &str,
    endpoint_name: &str,
    command_args: &[String],
) -> io::Result<u32> {
    let mut command = Command::new(env::current_exe()?);
    command
        .arg("--__tmux-server")
        .arg("--__session-name")
        .arg(session_name)
        .arg("--__endpoint-name")
        .arg(endpoint_name)
        .args(command_args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .envs(terman_common::terminal_env());

    let child = command.spawn()?;
    Ok(child.id())
}

#[cfg(test)]
mod tests {
    #[test]
    fn models_server_launcher_session_name() {
        assert_eq!("dev", "dev");
    }
}