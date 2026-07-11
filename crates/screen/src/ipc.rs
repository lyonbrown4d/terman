use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::region_types::{ScreenRegionAxis, ScreenRegionFocus};

mod endpoint;
pub(crate) use self::endpoint::ScreenIpcEndpoint;

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
    Hardcopy {
        include_history: bool,
    },
    SetHardcopyDir {
        path: PathBuf,
    },
    SetHardcopyAppend {
        append: bool,
    },
    Info,
    LastMessage,
    NewWindow {
        command: Option<String>,
    },
    SetDefaultCwd {
        path: PathBuf,
    },
    SetEnv {
        name: String,
        value: String,
    },
    UnsetEnv {
        name: String,
    },
    SelectWindow {
        index: usize,
    },
    NumberWindow {
        source: usize,
        index: usize,
    },
    NextWindow,
    PreviousWindow,
    LastWindow,
    SplitRegion {
        axis: ScreenRegionAxis,
    },
    FocusRegion {
        target: ScreenRegionFocus,
    },
    RemoveRegion,
    OnlyRegion,
    GetPasteBuffer,
    PasteBuffer,
    KillWindow,
    Ping,
    Quit,
    RenameSession {
        name: String,
    },
    Reset,
    Redisplay,
    SetLogEnabled {
        enabled: bool,
    },
    ToggleLog,
    SetLogFile {
        path: String,
    },
    SetLogFlush {
        seconds: u64,
    },
    SetLogTimestampEnabled {
        enabled: bool,
    },
    ToggleLogTimestamp,
    SetLogTimestampAfter {
        seconds: u64,
    },
    SetLogTimestampString {
        value: String,
    },
    SetPasteBuffer {
        bytes: Vec<u8>,
    },
    SetBufferFile {
        path: PathBuf,
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
    SetDefaultScrollback {
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
        hardcopy_dir: Option<PathBuf>,
        hardcopy_append: bool,
        buffer_file: PathBuf,
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



