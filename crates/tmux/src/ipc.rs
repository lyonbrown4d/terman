use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    io,
};

use interprocess::local_socket::{GenericNamespaced, Name, ToNsName};

const IPC_PREFIX: &str = "terman-tmux";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TmuxIpcEndpoint {
    raw_name: String,
}

impl TmuxIpcEndpoint {
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
    use super::TmuxIpcEndpoint;

    #[test]
    fn creates_stable_endpoint_name_for_session() {
        let left = TmuxIpcEndpoint::for_session("dev/session");
        let right = TmuxIpcEndpoint::for_session("dev/session");

        assert_eq!(left.raw_name(), right.raw_name());
        assert!(left.raw_name().starts_with("terman-tmux-dev_session-"));
    }
}
