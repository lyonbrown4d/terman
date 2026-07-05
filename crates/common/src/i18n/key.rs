#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MessageKey {
    NativeToolNotFound,
    BuiltinTmuxNoSessions,
    BuiltinTmuxSessionListEntry,
    BuiltinTmuxSessionCreated,
    BuiltinTmuxWindowCreated,
    BuiltinTmuxSessionExists,
    BuiltinTmuxSessionKilled,
    BuiltinTmuxSessionNameRequired,
    BuiltinTmuxSessionNotFound,
    BuiltinTmuxTargetRequired,
    BuiltinTmuxCommandUnsupported,
    BuiltinScreenNoSessions,
    BuiltinScreenSessionListHeader,
    BuiltinScreenSessionExists,
    BuiltinScreenSessionNameEmpty,
    BuiltinScreenAttachUnsupported,
    BuiltinScreenAttachHelp,
    BuiltinScreenAttachTargetRequired,
    BuiltinScreenSessionNotFound,
    BuiltinScreenNamedSessionRequired,
    BuiltinScreenServerTimeout,
    BuiltinScreenControlCommandRequired,
    BuiltinScreenControlCommandUnsupported,
    BuiltinScreenControlEchoRequired,
    BuiltinScreenControlStuffRequired,
    BuiltinScreenControlResizeRequired,
    BuiltinScreenControlInfo,
    BuiltinScreenControlHardcopyPathRequired,
    BuiltinScreenControlHardcopyComplete,
    BuiltinScreenControlPastefilePathRequired,
    BuiltinScreenWipeComplete,
}

impl MessageKey {
    pub(super) fn fluent_id(self) -> &'static str {
        match self {
            Self::NativeToolNotFound => "native-tool-not-found",
            Self::BuiltinTmuxNoSessions => "builtin-tmux-no-sessions",
            Self::BuiltinTmuxSessionListEntry => "builtin-tmux-session-list-entry",
            Self::BuiltinTmuxSessionCreated => "builtin-tmux-session-created",
            Self::BuiltinTmuxWindowCreated => "builtin-tmux-window-created",
            Self::BuiltinTmuxSessionExists => "builtin-tmux-session-exists",
            Self::BuiltinTmuxSessionKilled => "builtin-tmux-session-killed",
            Self::BuiltinTmuxSessionNameRequired => "builtin-tmux-session-name-required",
            Self::BuiltinTmuxSessionNotFound => "builtin-tmux-session-not-found",
            Self::BuiltinTmuxTargetRequired => "builtin-tmux-target-required",
            Self::BuiltinTmuxCommandUnsupported => "builtin-tmux-command-unsupported",
            Self::BuiltinScreenNoSessions => "builtin-screen-no-sessions",
            Self::BuiltinScreenSessionListHeader => "builtin-screen-session-list-header",
            Self::BuiltinScreenSessionExists => "builtin-screen-session-exists",
            Self::BuiltinScreenSessionNameEmpty => "builtin-screen-session-name-empty",
            Self::BuiltinScreenAttachUnsupported => "builtin-screen-attach-unsupported",
            Self::BuiltinScreenAttachHelp => "builtin-screen-attach-help",
            Self::BuiltinScreenAttachTargetRequired => "builtin-screen-attach-target-required",
            Self::BuiltinScreenSessionNotFound => "builtin-screen-session-not-found",
            Self::BuiltinScreenNamedSessionRequired => "builtin-screen-named-session-required",
            Self::BuiltinScreenServerTimeout => "builtin-screen-server-timeout",
            Self::BuiltinScreenControlCommandRequired => "builtin-screen-control-command-required",
            Self::BuiltinScreenControlCommandUnsupported => {
                "builtin-screen-control-command-unsupported"
            }
            Self::BuiltinScreenControlEchoRequired => "builtin-screen-control-echo-required",
            Self::BuiltinScreenControlStuffRequired => "builtin-screen-control-stuff-required",
            Self::BuiltinScreenControlResizeRequired => "builtin-screen-control-resize-required",
            Self::BuiltinScreenControlInfo => "builtin-screen-control-info",
            Self::BuiltinScreenControlHardcopyPathRequired => {
                "builtin-screen-control-hardcopy-path-required"
            }
            Self::BuiltinScreenControlHardcopyComplete => {
                "builtin-screen-control-hardcopy-complete"
            }
            Self::BuiltinScreenControlPastefilePathRequired => {
                "builtin-screen-control-pastefile-path-required"
            }
            Self::BuiltinScreenWipeComplete => "builtin-screen-wipe-complete",
        }
    }
}

