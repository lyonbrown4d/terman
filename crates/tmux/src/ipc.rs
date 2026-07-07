use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io, process,
    time::{SystemTime, UNIX_EPOCH},
};

use interprocess::local_socket::{
    ConnectOptions, GenericNamespaced, ListenerOptions, Name, ToNsName,
};
use serde::{Deserialize, Serialize};

const IPC_PREFIX: &str = "terman-tmux";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TmuxIpcEndpoint {
    raw_name: String,
}

impl TmuxIpcEndpoint {
    #[allow(dead_code)]
    pub(crate) fn from_raw_name(raw_name: &str) -> Self {
        Self {
            raw_name: raw_name.to_string(),
        }
    }

    pub(crate) fn for_new_session(session_name: &str) -> Self {
        let sanitized = sanitize_ipc_component(session_name);
        let entropy = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let pid = process::id();

        Self {
            raw_name: format!("{IPC_PREFIX}-{sanitized}-{pid:x}-{entropy:x}"),
        }
    }

    pub(crate) fn for_session(session_name: &str) -> Self {
        let mut hasher = DefaultHasher::new();
        session_name.hash(&mut hasher);
        let suffix = hasher.finish();
        let sanitized = sanitize_ipc_component(session_name);

        Self {
            raw_name: format!("{IPC_PREFIX}-{sanitized}-{suffix:016x}"),
        }
    }

    pub(crate) fn raw_name(&self) -> &str {
        &self.raw_name
    }

    pub(crate) fn socket_name(&self) -> io::Result<Name<'static>> {
        Ok(self
            .raw_name
            .clone()
            .to_ns_name::<GenericNamespaced>()?
            .into_owned())
    }

    #[allow(dead_code)]
    pub(crate) fn connect_options(&self) -> io::Result<ConnectOptions<'static>> {
        Ok(ConnectOptions::new().name(self.socket_name()?))
    }

    #[allow(dead_code)]
    pub(crate) fn listener_options(&self) -> io::Result<ListenerOptions<'static>> {
        Ok(ListenerOptions::new().name(self.socket_name()?))
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) enum TmuxIpcRequest {
    Attach { client_id: Option<String> },
    Detach,
    DetachClient { client_id: String },
    DetachAll,
    CapturePane,
    DisplayMessage { message: String },
    Info,
    Input { bytes: Vec<u8> },
    Ping,
    Quit,
    RenameSession { name: String },
    RenameWindow { index: u32, name: String },
    UpdateWindows { windows: u32 },
    NewWindow { index: u32, name: String, command: Option<String> },
    KillWindow { index: u32 },
    SelectWindow { index: u32 },
    Resize { cols: u16, rows: u16 },
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) enum TmuxIpcResponse {
    Accepted,
    Attached { replay: Vec<u8> },
    Detached,
    Captured { bytes: Vec<u8> },
    Info {
        session_name: String,
        windows: u32,
        attached_clients: u32,
        active_window: u32,
        window_indexes: Vec<u32>,
        window_names: Vec<String>,
        cwd: String,
    },
    Output { bytes: Vec<u8> },
    Rejected { reason: String },
    Resize { cols: u16, rows: u16 },
    Exit { code: i32 },
}

fn sanitize_ipc_component(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
                ch
            } else {
                '_'
            }
        })
        .collect();

    if sanitized.is_empty() {
        String::from("session")
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::{TmuxIpcEndpoint, TmuxIpcRequest, TmuxIpcResponse};

    #[test]
    fn creates_stable_endpoint_name_for_session() {
        let left = TmuxIpcEndpoint::for_session("dev/session");
        let right = TmuxIpcEndpoint::for_session("dev/session");

        assert_eq!(left.raw_name(), right.raw_name());
        assert!(left.raw_name().starts_with("terman-tmux-dev_session-"));
    }

    #[test]
    fn creates_unique_endpoint_name_for_new_session() {
        let endpoint = TmuxIpcEndpoint::for_new_session("dev/session");

        assert!(endpoint.raw_name().starts_with("terman-tmux-dev_session-"));
    }

    #[test]
    fn preserves_raw_endpoint_name() {
        let endpoint = TmuxIpcEndpoint::from_raw_name("terman-tmux-dev");

        assert_eq!(endpoint.raw_name(), "terman-tmux-dev");
    }

    #[test]
    fn models_tmux_ipc_protocol() {
        assert_eq!(TmuxIpcRequest::Ping, TmuxIpcRequest::Ping);
        assert_eq!(
            TmuxIpcResponse::Info {
                session_name: String::from("dev"),
                windows: 1,
                attached_clients: 0,
                active_window: 0,
                window_indexes: vec![0],
                window_names: vec![String::from("0")],
                cwd: String::from("/tmp"),
            },
            TmuxIpcResponse::Info {
                session_name: String::from("dev"),
                windows: 1,
                attached_clients: 0,
                active_window: 0,
                window_indexes: vec![0],
                window_names: vec![String::from("0")],
                cwd: String::from("/tmp"),
            }
        );
    }
}