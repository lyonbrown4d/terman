#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MessageKey {
    NativeToolNotFound,
    BuiltinTmuxNoSessions,
    BuiltinTmuxCliAbout,
    BuiltinTmuxCliExamples,
    BuiltinTmuxSessionListEntry,
    BuiltinTmuxClientListEntry,
    BuiltinTmuxWindowListEntry,
    BuiltinTmuxSessionCreated,
    BuiltinTmuxWindowCreated,
    BuiltinTmuxWindowKilled,
    BuiltinTmuxWindowNameRequired,
    BuiltinTmuxWindowNotFound,
    BuiltinTmuxSessionExists,
    BuiltinTmuxSessionKilled,
    BuiltinTmuxSessionNameRequired,
    BuiltinTmuxSessionNotFound,
    BuiltinTmuxTargetRequired,
    BuiltinTmuxCommandUnsupported,
    BuiltinTmuxInternalServerSessionRequired,
    BuiltinTmuxInternalServerExited,
    BuiltinTmuxServerNotResponding,
    BuiltinTmuxServerNotReady,
    BuiltinTmuxUnexpectedInfoResponse,
    BuiltinTmuxUnexpectedResponse,
    BuiltinTmuxMessageRequired,
    BuiltinTmuxKeysRequired,
    BuiltinScreenNoSessions,
    BuiltinScreenCliAbout,
    BuiltinScreenCliExamples,
    BuiltinScreenSessionListHeader,
    BuiltinScreenSessionListEntry,
    BuiltinScreenSessionExists,
    BuiltinScreenSessionNameEmpty,
    BuiltinScreenSessionRecordInvalid,
    BuiltinScreenUnexpectedResponse,
    BuiltinScreenAttachUnsupported,
    BuiltinScreenAttachHelp,
    BuiltinScreenAttachHardcopyPathUnavailable,
    BuiltinScreenAttachTargetRequired,
    BuiltinScreenAttachOutputThreadPanicked,
    BuiltinScreenSessionNotFound,
    BuiltinScreenNamedSessionRequired,
    BuiltinScreenServerTimeout,
    BuiltinScreenServiceTimeout,
    BuiltinScreenInternalServerSessionRequired,
    BuiltinScreenInternalServerExited,
    BuiltinScreenFailure,
    BuiltinScreenControlCommandRequired,
    BuiltinScreenControlCommandUnsupported,
    BuiltinScreenControlEchoRequired,
    BuiltinScreenControlLogRequired,
    BuiltinScreenControlLogfileRequired,
    BuiltinScreenControlStuffRequired,
    BuiltinScreenControlResizeRequired,
    BuiltinScreenControlHelp,
    BuiltinScreenControlSelectUnsupported,
    BuiltinScreenControlScrollbackRequired,
    BuiltinScreenControlSleepRequired,
    BuiltinScreenControlTime,
    BuiltinScreenControlTitleRequired,
    BuiltinScreenControlVersion,
    BuiltinScreenControlInfo,
    BuiltinScreenControlDisplaysEntry,
    BuiltinScreenControlWindowsEntry,
    BuiltinScreenControlUnexpectedResponse,
    BuiltinScreenControlHardcopyPathRequired,
    BuiltinScreenControlHardcopyComplete,
    BuiltinScreenControlPastefilePathRequired,
    BuiltinScreenControlReadbufPathRequired,
    BuiltinScreenControlSourcePathRequired,
    BuiltinScreenWipeComplete,
}

impl MessageKey {
    pub(crate) fn fluent_id(self) -> &'static str {
        match self {
            Self::NativeToolNotFound => "native-tool-not-found",
            Self::BuiltinTmuxNoSessions => "builtin-tmux-no-sessions",
            Self::BuiltinTmuxCliAbout => "builtin-tmux-cli-about",
            Self::BuiltinTmuxCliExamples => "builtin-tmux-cli-examples",
            Self::BuiltinTmuxSessionListEntry => "builtin-tmux-session-list-entry",
            Self::BuiltinTmuxClientListEntry => "builtin-tmux-client-list-entry",
            Self::BuiltinTmuxWindowListEntry => "builtin-tmux-window-list-entry",
            Self::BuiltinTmuxSessionCreated => "builtin-tmux-session-created",
            Self::BuiltinTmuxWindowCreated => "builtin-tmux-window-created",
            Self::BuiltinTmuxWindowKilled => "builtin-tmux-window-killed",
            Self::BuiltinTmuxWindowNameRequired => "builtin-tmux-window-name-required",
            Self::BuiltinTmuxWindowNotFound => "builtin-tmux-window-not-found",
            Self::BuiltinTmuxSessionExists => "builtin-tmux-session-exists",
            Self::BuiltinTmuxSessionKilled => "builtin-tmux-session-killed",
            Self::BuiltinTmuxSessionNameRequired => "builtin-tmux-session-name-required",
            Self::BuiltinTmuxSessionNotFound => "builtin-tmux-session-not-found",
            Self::BuiltinTmuxTargetRequired => "builtin-tmux-target-required",
            Self::BuiltinTmuxCommandUnsupported => "builtin-tmux-command-unsupported",
            Self::BuiltinTmuxInternalServerSessionRequired => {
                "builtin-tmux-internal-server-session-required"
            }
            Self::BuiltinTmuxInternalServerExited => "builtin-tmux-internal-server-exited",
            Self::BuiltinTmuxServerNotResponding => "builtin-tmux-server-not-responding",
            Self::BuiltinTmuxServerNotReady => "builtin-tmux-server-not-ready",
            Self::BuiltinTmuxUnexpectedInfoResponse => "builtin-tmux-unexpected-info-response",
            Self::BuiltinTmuxUnexpectedResponse => "builtin-tmux-unexpected-response",
            Self::BuiltinTmuxMessageRequired => "builtin-tmux-message-required",
            Self::BuiltinTmuxKeysRequired => "builtin-tmux-keys-required",
            Self::BuiltinScreenNoSessions => "builtin-screen-no-sessions",
            Self::BuiltinScreenCliAbout => "builtin-screen-cli-about",
            Self::BuiltinScreenCliExamples => "builtin-screen-cli-examples",
            Self::BuiltinScreenSessionListHeader => "builtin-screen-session-list-header",
            Self::BuiltinScreenSessionListEntry => "builtin-screen-session-list-entry",
            Self::BuiltinScreenSessionExists => "builtin-screen-session-exists",
            Self::BuiltinScreenSessionNameEmpty => "builtin-screen-session-name-empty",
            Self::BuiltinScreenSessionRecordInvalid => "builtin-screen-session-record-invalid",
            Self::BuiltinScreenUnexpectedResponse => "builtin-screen-unexpected-response",
            Self::BuiltinScreenAttachUnsupported => "builtin-screen-attach-unsupported",
            Self::BuiltinScreenAttachHelp => "builtin-screen-attach-help",
            Self::BuiltinScreenAttachHardcopyPathUnavailable => {
                "builtin-screen-attach-hardcopy-path-unavailable"
            }
            Self::BuiltinScreenAttachTargetRequired => "builtin-screen-attach-target-required",
            Self::BuiltinScreenAttachOutputThreadPanicked => {
                "builtin-screen-attach-output-thread-panicked"
            }
            Self::BuiltinScreenSessionNotFound => "builtin-screen-session-not-found",
            Self::BuiltinScreenNamedSessionRequired => "builtin-screen-named-session-required",
            Self::BuiltinScreenServerTimeout => "builtin-screen-server-timeout",
            Self::BuiltinScreenServiceTimeout => "builtin-screen-service-timeout",
            Self::BuiltinScreenInternalServerSessionRequired => {
                "builtin-screen-internal-server-session-required"
            }
            Self::BuiltinScreenInternalServerExited => "builtin-screen-internal-server-exited",
            Self::BuiltinScreenFailure => "builtin-screen-failure",
            Self::BuiltinScreenControlCommandRequired => "builtin-screen-control-command-required",
            Self::BuiltinScreenControlCommandUnsupported => {
                "builtin-screen-control-command-unsupported"
            }
            Self::BuiltinScreenControlEchoRequired => "builtin-screen-control-echo-required",
            Self::BuiltinScreenControlLogRequired => "builtin-screen-control-log-required",
            Self::BuiltinScreenControlLogfileRequired => {
                "builtin-screen-control-logfile-required"
            }
            Self::BuiltinScreenControlStuffRequired => "builtin-screen-control-stuff-required",
            Self::BuiltinScreenControlResizeRequired => "builtin-screen-control-resize-required",
            Self::BuiltinScreenControlHelp => "builtin-screen-control-help",
            Self::BuiltinScreenControlSelectUnsupported => "builtin-screen-control-select-unsupported",
            Self::BuiltinScreenControlScrollbackRequired => {
                "builtin-screen-control-scrollback-required"
            }
            Self::BuiltinScreenControlSleepRequired => "builtin-screen-control-sleep-required",
            Self::BuiltinScreenControlTime => "builtin-screen-control-time",
            Self::BuiltinScreenControlTitleRequired => "builtin-screen-control-title-required",
            Self::BuiltinScreenControlVersion => "builtin-screen-control-version",
            Self::BuiltinScreenControlInfo => "builtin-screen-control-info",
            Self::BuiltinScreenControlDisplaysEntry => "builtin-screen-control-displays-entry",
            Self::BuiltinScreenControlWindowsEntry => "builtin-screen-control-windows-entry",
            Self::BuiltinScreenControlUnexpectedResponse => {
                "builtin-screen-control-unexpected-response"
            }
            Self::BuiltinScreenControlHardcopyPathRequired => {
                "builtin-screen-control-hardcopy-path-required"
            }
            Self::BuiltinScreenControlHardcopyComplete => {
                "builtin-screen-control-hardcopy-complete"
            }
            Self::BuiltinScreenControlPastefilePathRequired => {
                "builtin-screen-control-pastefile-path-required"
            }
            Self::BuiltinScreenControlReadbufPathRequired => {
                "builtin-screen-control-readbuf-path-required"
            }
            Self::BuiltinScreenControlSourcePathRequired => {
                "builtin-screen-control-source-path-required"
            }
            Self::BuiltinScreenWipeComplete => "builtin-screen-wipe-complete",
        }
    }
}
