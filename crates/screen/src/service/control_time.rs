use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn request_time_command() {
    println!("{}", screen_time_message());
}

pub(super) fn screen_time_message() -> String {
    terman_common::builtin_screen_control_time_hint(current_unix_seconds())
}

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}