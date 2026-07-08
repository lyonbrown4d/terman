use std::io;

use serde::{Deserialize, Serialize};
use sysinfo::{Pid, System};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct BuiltinScreenSession {
    pub(crate) name: String,
    pub(crate) pid: String,
    pub(crate) cwd: String,
    pub(crate) command: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) ipc_endpoint: Option<String>,
}

pub(crate) fn builtin_screen_session_is_alive(
    session: &BuiltinScreenSession,
    system: &System,
) -> bool {
    session
        .pid
        .parse::<u32>()
        .ok()
        .map(|pid| system.process(Pid::from_u32(pid)).is_some())
        .unwrap_or(false)
}

pub(crate) fn parse_builtin_screen_session_record(record: &str) -> Option<BuiltinScreenSession> {
    serde_json::from_str(record).ok()
}

pub(super) fn serialize_session_record(session: &BuiltinScreenSession) -> io::Result<String> {
    serde_json::to_string_pretty(session)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}