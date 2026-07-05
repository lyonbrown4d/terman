use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct BuiltinTmuxSession {
    pub(crate) name: String,
    #[serde(default = "default_window_count")]
    pub(crate) windows: u32,
    #[serde(default)]
    pub(crate) attached_clients: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RenameBuiltinTmuxSession {
    Renamed,
    SourceMissing,
    DestinationExists,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AddBuiltinTmuxWindow {
    Added(u32),
    SessionMissing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KillBuiltinTmuxWindow {
    Killed(u32),
    SessionKilled,
    SessionMissing,
}

pub(crate) fn parse_builtin_tmux_session_record(record: &str) -> Option<BuiltinTmuxSession> {
    serde_json::from_str(record).ok()
}

fn default_window_count() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::{BuiltinTmuxSession, parse_builtin_tmux_session_record};

    #[test]
    fn parses_tmux_session_record_with_defaults() {
        let session = parse_builtin_tmux_session_record(r#"{"name":"dev"}"#).unwrap();

        assert_eq!(
            session,
            BuiltinTmuxSession {
                name: String::from("dev"),
                windows: 1,
                attached_clients: 0,
            }
        );
    }
}
