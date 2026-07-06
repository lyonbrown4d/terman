use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io, process,
    time::{SystemTime, UNIX_EPOCH},
};

use interprocess::local_socket::{
    ConnectOptions, GenericNamespaced, ListenerOptions, Name, ToNsName,
};

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