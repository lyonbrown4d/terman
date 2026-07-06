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

const IPC_PREFIX: &str = "terman-screen";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScreenIpcEndpoint {
    raw_name: String,
}

impl ScreenIpcEndpoint {
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

    pub(crate) fn listener_options(&self) -> io::Result<ListenerOptions<'static>> {
        Ok(ListenerOptions::new().name(self.socket_name()?))
    }

    pub(crate) fn connect_options(&self) -> io::Result<ConnectOptions<'static>> {
        Ok(ConnectOptions::new().name(self.socket_name()?))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) enum ScreenAttachMode {
    Resume,
    MultiAttach,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct ScreenWindowInfo {
    pub(crate) index: usize,
    pub(crate) title: String,
    pub(crate) active: bool,
    pub(crate) replay_bytes: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) enum ScreenIpcRequest {
    Attach {
        mode: ScreenAttachMode,
        target: Option<String>,
        detach_existing: bool,
        client_id: Option<String>,
    },
    Detach,
    DetachClient {
        client_id: String,
    },
    DetachAll,
    Bell,
    Clear,
    Echo {
        message: String,
    },
    Hardcopy,
    Info,
    NewWindow {
        command: Option<String>,
    },
    SelectWindow {
        index: usize,
    },
    NextWindow,
    PreviousWindow,
    LastWindow,
    GetPasteBuffer,
    PasteBuffer,
    KillWindow,
    Ping,
    Quit,
    RenameSession {
        name: String,
    },
    Reset,
    SetLogEnabled {
        enabled: bool,
    },
    SetLogFile {
        path: String,
    },
    SetPasteBuffer {
        bytes: Vec<u8>,
    },
    SetRegister {
        name: String,
        bytes: Vec<u8>,
    },
    PasteRegister {
        name: String,
    },
    SetScrollback {
        lines: usize,
    },
    SetWindowTitle {
        title: String,
    },
    Input {
        bytes: Vec<u8>,
    },
    Resize {
        cols: u16,
        rows: u16,
    },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) enum ScreenIpcResponse {
    Accepted,
    Attached { replay: Vec<u8> },
    Detached,
    Hardcopy { bytes: Vec<u8> },
    Info {
        session_name: String,
        replay_bytes: usize,
        attach_clients: usize,
        cols: Option<u16>,
        rows: Option<u16>,
        scrollback_lines: usize,
        window_title: Option<String>,
        active_window: usize,
        windows: Vec<ScreenWindowInfo>,
    },
    Output { bytes: Vec<u8> },
    PasteBuffer { bytes: Vec<u8> },
    Resize { cols: u16, rows: u16 },
    Exit { code: i32 },
    Rejected { reason: String },
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
    use super::{ScreenAttachMode, ScreenIpcEndpoint, ScreenIpcRequest, ScreenIpcResponse};

    #[test]
    fn creates_stable_endpoint_name_for_session() {
        let left = ScreenIpcEndpoint::for_session("dev/session");
        let right = ScreenIpcEndpoint::for_session("dev/session");

        assert_eq!(left.raw_name(), right.raw_name());
        assert!(left.raw_name().starts_with("terman-screen-dev_session-"));
    }

    #[test]
    fn creates_unique_endpoint_name_for_new_session() {
        let endpoint = ScreenIpcEndpoint::for_new_session("dev/session");

        assert!(endpoint.raw_name().starts_with("terman-screen-dev_session-"));
    }
    #[test]
    fn preserves_raw_endpoint_name_from_session_record() {
        let endpoint = ScreenIpcEndpoint::from_raw_name("terman-screen-dev");

        assert_eq!(endpoint.raw_name(), "terman-screen-dev");
    }

    #[test]
    fn models_attach_request_protocol() {
        let request = ScreenIpcRequest::Attach {
            mode: ScreenAttachMode::Resume,
            target: Some(String::from("dev")),
            detach_existing: false,
            client_id: Some(String::from("client")),
        };

        assert_eq!(
            request,
            ScreenIpcRequest::Attach {
                mode: ScreenAttachMode::Resume,
                target: Some(String::from("dev")),
                detach_existing: false,
                client_id: Some(String::from("client")),
            }
        );
    }

    #[test]
    fn models_attach_stream_responses() {
        assert_eq!(
            ScreenIpcResponse::Attached {
                replay: b"hello".to_vec()
            },
            ScreenIpcResponse::Attached {
                replay: b"hello".to_vec()
            }
        );
        assert_eq!(
            ScreenIpcResponse::Output {
                bytes: b"x".to_vec()
            },
            ScreenIpcResponse::Output {
                bytes: b"x".to_vec()
            }
        );
    }
}



