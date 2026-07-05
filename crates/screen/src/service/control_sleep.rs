use std::{io, thread, time::Duration};

use super::control_parse::control_command_payload;

pub(super) fn request_sleep_command(inline_payload: &str, extra_args: &[String]) -> io::Result<()> {
    let seconds = sleep_seconds(&control_command_payload(inline_payload, extra_args))?;
    thread::sleep(Duration::from_secs(seconds));
    Ok(())
}

fn sleep_seconds(payload: &str) -> io::Result<u64> {
    let mut parts = payload.split_whitespace();
    let Some(seconds) = parts.next().and_then(|value| value.parse::<u64>().ok()) else {
        return Err(sleep_required_error());
    };
    if parts.next().is_some() {
        return Err(sleep_required_error());
    }
    Ok(seconds)
}

fn sleep_required_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        terman_common::builtin_screen_control_sleep_required_hint(),
    )
}

#[cfg(test)]
mod tests {
    use super::sleep_seconds;

    #[test]
    fn parses_sleep_seconds() {
        assert_eq!(sleep_seconds("0").unwrap(), 0);
        assert_eq!(sleep_seconds("3").unwrap(), 3);
        assert!(sleep_seconds("").is_err());
        assert!(sleep_seconds("1 2").is_err());
        assert!(sleep_seconds("1.5").is_err());
    }
}