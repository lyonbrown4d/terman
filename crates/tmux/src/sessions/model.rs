use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct BuiltinTmuxSession {
    pub(crate) name: String,
    #[serde(default = "default_window_count")]
    pub(crate) windows: u32,
    #[serde(default)]
    pub(crate) attached_clients: u32,
    #[serde(default = "default_cwd")]
    pub(crate) cwd: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) pid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) ipc_endpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) window_indexes: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) window_names: Vec<String>,
}

impl BuiltinTmuxSession {
    pub(crate) fn window_indices(&self) -> Vec<u32> {
        let mut indexes = if self.window_indexes.is_empty() {
            (0..self.windows.max(1)).collect()
        } else {
            self.window_indexes.clone()
        };
        indexes.sort_unstable();
        indexes.dedup();
        indexes
    }

    pub(crate) fn window_name(&self, index: u32) -> String {
        self.window_indices()
            .iter()
            .position(|candidate| *candidate == index)
            .and_then(|position| self.window_names.get(position).cloned())
            .unwrap_or_else(|| index.to_string())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RenameBuiltinTmuxSession {
    Renamed,
    SourceMissing,
    DestinationExists,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AddBuiltinTmuxWindow {
    Added { windows: u32, index: u32, name: String },
    SessionMissing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum KillBuiltinTmuxWindow {
    Killed { windows: u32, index: u32 },
    SessionKilled,
    SessionMissing,
    WindowMissing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RenameBuiltinTmuxWindow {
    Renamed,
    SessionMissing,
    WindowMissing,
}

pub(crate) fn parse_builtin_tmux_session_record(record: &str) -> Option<BuiltinTmuxSession> {
    serde_json::from_str(record).ok()
}

fn default_window_count() -> u32 {
    1
}

fn default_cwd() -> String {
    String::from("<unknown>")
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
                cwd: String::from("<unknown>"),
                command: None,
                pid: None,
                ipc_endpoint: None,
                window_indexes: Vec::new(),
                window_names: Vec::new(),
            }
        );
    }

    #[test]
    fn returns_default_window_name() {
        let session = BuiltinTmuxSession {
            name: String::from("dev"),
            windows: 1,
            attached_clients: 0,
            cwd: String::from("<unknown>"),
            command: None,
            pid: None,
            ipc_endpoint: None,
            window_indexes: Vec::new(),
            window_names: Vec::new(),
        };

        assert_eq!(session.window_name(0), "0");
    }
}