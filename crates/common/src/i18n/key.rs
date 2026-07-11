#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MessageKey {
    NativeToolNotFound,
    BuiltinHtopCliAbout,
    BuiltinHtopTabOverview,
    BuiltinHtopTabProcesses,
    BuiltinHtopTabIo,
    BuiltinHtopTabNetwork,
    BuiltinHtopHelp,
    BuiltinHtopHelpPanel, BuiltinHtopTreeCollapse, BuiltinHtopTreeExpand, BuiltinHtopTreeToggleAll, BuiltinHtopTag, BuiltinHtopUntagAll, BuiltinHtopTaggedCount,
    BuiltinHtopSortMenuTitle, BuiltinHtopUserFilter, BuiltinHtopAllUsers, BuiltinHtopUserMenuTitle, BuiltinHtopUserMenuHelp,
    BuiltinHtopSortMenuHelp,
    BuiltinHtopSignalMenuTitle,
    BuiltinHtopSignalMenuHelp,
    BuiltinHtopSignalUnsupported,
    BuiltinHtopSignalFooter, BuiltinHtopFollowStatus, BuiltinHtopSetupTitle, BuiltinHtopSetupHelp, BuiltinHtopSetupRefresh, BuiltinHtopSetupTree, BuiltinHtopSetupSortDirection, BuiltinHtopSetupEnabled, BuiltinHtopSetupDisabled, BuiltinHtopSetupAscending, BuiltinHtopSetupDescending,
    BuiltinHtopFooterHelp, BuiltinHtopFooterSearch, BuiltinHtopFooterFilter, BuiltinHtopFooterPriorityHigher, BuiltinHtopFooterPriorityLower, BuiltinHtopFooterKill, BuiltinHtopFooterDelay, BuiltinHtopFooterQuit, BuiltinHtopFooterYes, BuiltinHtopFooterNo, BuiltinHtopFooterSearchPrompt, BuiltinHtopFooterFilterPrompt, BuiltinHtopFooterTreePrompt, BuiltinHtopFooterSelectPrompt, BuiltinHtopFooterViewFlat, BuiltinHtopOverviewCpu, BuiltinHtopOverviewHost, BuiltinHtopOverviewKernel, BuiltinHtopOverviewTasks, BuiltinHtopOverviewStates, BuiltinHtopOverviewNetwork, BuiltinHtopOverviewUptime, BuiltinHtopOverviewLoad, BuiltinHtopOverviewTopProcesses, BuiltinHtopDetailNone, BuiltinHtopDetailUser, BuiltinHtopDetailStatus, BuiltinHtopDetailMemory, BuiltinHtopDetailRuntime, BuiltinHtopDetailRead, BuiltinHtopDetailWrite, BuiltinHtopDetailCommand, BuiltinHtopDetailIo, BuiltinHtopProcessesTitle, BuiltinHtopProcessesDetails, BuiltinHtopProcessesStatus, BuiltinHtopProcessesViewTree, BuiltinHtopProcessesViewFlat, BuiltinScreenConfirmKill, BuiltinScreenConfirmQuit, BuiltinTmuxFindPrompt, BuiltinTmuxFindNoMatch, BuiltinHtopEnvironmentTitle, BuiltinHtopEnvironmentHelp, BuiltinHtopEnvironmentEmpty,
    BuiltinTmuxNoSessions,
    BuiltinTmuxCliAbout,
    BuiltinTmuxCliExamples,
    BuiltinTmuxAttachHelp,
    BuiltinTmuxCopyStatus,
    BuiltinTmuxCopySelectionStatus,
    BuiltinTmuxBufferDataRequired, BuiltinTmuxBufferListItem,
    BuiltinTmuxBufferNotFound,
    BuiltinTmuxBufferUnavailable,
    BuiltinTmuxPrefixStatus, BuiltinTmuxRenameSessionPrompt, BuiltinTmuxRenameWindowPrompt, BuiltinTmuxCommandPrompt, BuiltinTmuxCommandParseError,
    BuiltinTmuxAttachWindowList, BuiltinTmuxPaneChooser, BuiltinTmuxStatusLine, BuiltinTmuxKillPaneConfirm, BuiltinTmuxKillWindowConfirm,
    BuiltinTmuxSessionListEntry,
    BuiltinTmuxClientListEntry,
    BuiltinTmuxWindowListEntry,
    BuiltinTmuxPaneListEntry,
    BuiltinTmuxPaneNotFound,
    BuiltinTmuxPaneSizeRequired, BuiltinTmuxWindowOptionUnsupported, BuiltinTmuxSynchronizePanesRequired,
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
    BuiltinScreenAttachHelp, BuiltinScreenWindowListStatus,
    BuiltinScreenCopyStatus, BuiltinScreenCopySelectionStatus, BuiltinScreenMonitorStatus, BuiltinScreenMonitorActivity, BuiltinScreenSilenceStatus, BuiltinScreenSilenceActivity, BuiltinScreenControlMonitorRequired, BuiltinScreenControlSilenceRequired, BuiltinScreenWrapStatus, BuiltinScreenControlWrapRequired,
    BuiltinScreenAttachHardcopyPathUnavailable,
    BuiltinScreenAttachTitlePrompt, BuiltinScreenAttachSelectPrompt, BuiltinScreenAttachCommandPrompt,
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
    BuiltinScreenControlChdirDirectoryRequired,
    BuiltinScreenControlChdirHomeRequired,
    BuiltinScreenControlEchoRequired,
    BuiltinScreenControlLastmsgEmpty,
    BuiltinScreenControlSetenvRequired,
    BuiltinScreenControlUnsetenvRequired,
    BuiltinScreenControlEnvNameInvalid,
    BuiltinScreenControlShellRequired,
    BuiltinScreenControlShelltitleRequired,
    BuiltinScreenControlTermRequired,
    BuiltinScreenControlLogRequired,
    BuiltinScreenControlLogfileRequired,
    BuiltinScreenControlLogtstampRequired,
    BuiltinScreenControlStuffRequired,
    BuiltinScreenControlRegisterRequired,
    BuiltinScreenControlResizeRequired,
    BuiltinScreenControlSizeRequired,
    BuiltinScreenControlHelp,
    BuiltinScreenControlSelectUnsupported,
    BuiltinScreenControlNumber,
    BuiltinScreenControlNumberInvalid,
    BuiltinScreenControlScrollbackRequired,
    BuiltinScreenControlSleepRequired,
    BuiltinScreenControlTime,
    BuiltinScreenControlTitleRequired,
    BuiltinScreenControlVersion,
    BuiltinScreenControlLicense,
    BuiltinScreenControlInfo,
    BuiltinScreenControlDinfo,
    BuiltinScreenControlDumptermcapComplete,
    BuiltinScreenControlDisplaysEntry,
    BuiltinScreenControlWindowsEntry,
    BuiltinScreenControlUnexpectedResponse,
    BuiltinScreenControlHardcopyPathRequired,
    BuiltinScreenControlHardcopydirRequired,
    BuiltinScreenControlHardcopyAppendRequired,
    BuiltinScreenControlHardcopyComplete,
    BuiltinScreenControlPastefilePathRequired,
    BuiltinScreenControlReadbufPathRequired,
    BuiltinScreenControlReadregRequired,
    BuiltinScreenControlSourcePathRequired,
    BuiltinScreenControlWritebufPathRequired,
    BuiltinScreenControlBufferEncodingRequired,
    BuiltinScreenControlWritebufComplete, BuiltinScreenControlReadbufComplete, BuiltinScreenControlRemovebufComplete, BuiltinScreenControlBufferIoError,
    BuiltinScreenWipeComplete,
}
impl MessageKey {
    pub(crate) fn fluent_id(self) -> &'static str {
        match self {
            Self::NativeToolNotFound => "native-tool-not-found",
            Self::BuiltinHtopCliAbout => "builtin-htop-cli-about",
            Self::BuiltinHtopTabOverview => "builtin-htop-tab-overview",
            Self::BuiltinHtopTabProcesses => "builtin-htop-tab-processes",
            Self::BuiltinHtopTabIo => "builtin-htop-tab-io",
            Self::BuiltinHtopTabNetwork => "builtin-htop-tab-network",
            Self::BuiltinHtopHelp => "builtin-htop-help",
            Self::BuiltinHtopHelpPanel => "builtin-htop-help-panel", Self::BuiltinHtopTreeCollapse => "builtin-htop-tree-collapse", Self::BuiltinHtopTreeExpand => "builtin-htop-tree-expand", Self::BuiltinHtopTreeToggleAll => "builtin-htop-tree-toggle-all", Self::BuiltinHtopTag => "builtin-htop-tag", Self::BuiltinHtopUntagAll => "builtin-htop-untag-all", Self::BuiltinHtopTaggedCount => "builtin-htop-tagged-count",
            Self::BuiltinHtopSortMenuTitle => "builtin-htop-sort-menu-title", Self::BuiltinHtopUserFilter => "builtin-htop-user-filter", Self::BuiltinHtopAllUsers => "builtin-htop-all-users", Self::BuiltinHtopUserMenuTitle => "builtin-htop-user-menu-title", Self::BuiltinHtopUserMenuHelp => "builtin-htop-user-menu-help",
            Self::BuiltinHtopSortMenuHelp => "builtin-htop-sort-menu-help",
            Self::BuiltinHtopSignalMenuTitle => "builtin-htop-signal-menu-title",
            Self::BuiltinHtopSignalMenuHelp => "builtin-htop-signal-menu-help",
            Self::BuiltinHtopSignalUnsupported => "builtin-htop-signal-unsupported",
            Self::BuiltinHtopSignalFooter => "builtin-htop-signal-footer", Self::BuiltinHtopFollowStatus => "builtin-htop-follow-status", Self::BuiltinHtopSetupTitle => "builtin-htop-setup-title", Self::BuiltinHtopSetupHelp => "builtin-htop-setup-help", Self::BuiltinHtopSetupRefresh => "builtin-htop-setup-refresh", Self::BuiltinHtopSetupTree => "builtin-htop-setup-tree", Self::BuiltinHtopSetupSortDirection => "builtin-htop-setup-sort-direction", Self::BuiltinHtopSetupEnabled => "builtin-htop-setup-enabled", Self::BuiltinHtopSetupDisabled => "builtin-htop-setup-disabled", Self::BuiltinHtopSetupAscending => "builtin-htop-setup-ascending", Self::BuiltinHtopSetupDescending => "builtin-htop-setup-descending",
            Self::BuiltinHtopFooterHelp => "builtin-htop-footer-help", Self::BuiltinHtopFooterSearch => "builtin-htop-footer-search", Self::BuiltinHtopFooterFilter => "builtin-htop-footer-filter", Self::BuiltinHtopFooterPriorityHigher => "builtin-htop-footer-priority-higher", Self::BuiltinHtopFooterPriorityLower => "builtin-htop-footer-priority-lower", Self::BuiltinHtopFooterKill => "builtin-htop-footer-kill", Self::BuiltinHtopFooterDelay => "builtin-htop-footer-delay", Self::BuiltinHtopFooterQuit => "builtin-htop-footer-quit", Self::BuiltinHtopFooterYes => "builtin-htop-footer-yes", Self::BuiltinHtopFooterNo => "builtin-htop-footer-no", Self::BuiltinHtopFooterSearchPrompt => "builtin-htop-footer-search-prompt", Self::BuiltinHtopFooterFilterPrompt => "builtin-htop-footer-filter-prompt", Self::BuiltinHtopFooterTreePrompt => "builtin-htop-footer-tree-prompt", Self::BuiltinHtopFooterSelectPrompt => "builtin-htop-footer-select-prompt", Self::BuiltinHtopFooterViewFlat => "builtin-htop-footer-view-flat", Self::BuiltinHtopOverviewCpu => "builtin-htop-overview-cpu", Self::BuiltinHtopOverviewHost => "builtin-htop-overview-host", Self::BuiltinHtopOverviewKernel => "builtin-htop-overview-kernel", Self::BuiltinHtopOverviewTasks => "builtin-htop-overview-tasks", Self::BuiltinHtopOverviewStates => "builtin-htop-overview-states", Self::BuiltinHtopOverviewNetwork => "builtin-htop-overview-network", Self::BuiltinHtopOverviewUptime => "builtin-htop-overview-uptime", Self::BuiltinHtopOverviewLoad => "builtin-htop-overview-load", Self::BuiltinHtopOverviewTopProcesses => "builtin-htop-overview-top-processes", Self::BuiltinHtopDetailNone => "builtin-htop-detail-none", Self::BuiltinHtopDetailUser => "builtin-htop-detail-user", Self::BuiltinHtopDetailStatus => "builtin-htop-detail-status", Self::BuiltinHtopDetailMemory => "builtin-htop-detail-memory", Self::BuiltinHtopDetailRuntime => "builtin-htop-detail-runtime", Self::BuiltinHtopDetailRead => "builtin-htop-detail-read", Self::BuiltinHtopDetailWrite => "builtin-htop-detail-write", Self::BuiltinHtopDetailCommand => "builtin-htop-detail-command", Self::BuiltinHtopDetailIo => "builtin-htop-detail-io", Self::BuiltinHtopProcessesTitle => "builtin-htop-processes-title", Self::BuiltinHtopProcessesDetails => "builtin-htop-processes-details", Self::BuiltinHtopProcessesStatus => "builtin-htop-processes-status", Self::BuiltinHtopProcessesViewTree => "builtin-htop-processes-view-tree", Self::BuiltinHtopProcessesViewFlat => "builtin-htop-processes-view-flat", Self::BuiltinScreenConfirmKill => "builtin-screen-confirm-kill", Self::BuiltinScreenConfirmQuit => "builtin-screen-confirm-quit", Self::BuiltinTmuxFindPrompt => "builtin-tmux-find-prompt", Self::BuiltinTmuxFindNoMatch => "builtin-tmux-find-no-match", Self::BuiltinHtopEnvironmentTitle => "builtin-htop-environment-title", Self::BuiltinHtopEnvironmentHelp => "builtin-htop-environment-help", Self::BuiltinHtopEnvironmentEmpty => "builtin-htop-environment-empty",
            Self::BuiltinTmuxNoSessions => "builtin-tmux-no-sessions",
            Self::BuiltinTmuxCliAbout => "builtin-tmux-cli-about",
            Self::BuiltinTmuxCliExamples => "builtin-tmux-cli-examples",
            Self::BuiltinTmuxAttachHelp => "builtin-tmux-attach-help",
            Self::BuiltinTmuxCopyStatus => "builtin-tmux-copy-status",
            Self::BuiltinTmuxCopySelectionStatus => "builtin-tmux-copy-selection-status",
            Self::BuiltinTmuxBufferDataRequired => "builtin-tmux-buffer-data-required",
            Self::BuiltinTmuxBufferListItem => "builtin-tmux-buffer-list-item",
            Self::BuiltinTmuxBufferNotFound => "builtin-tmux-buffer-not-found",
            Self::BuiltinTmuxBufferUnavailable => "builtin-tmux-buffer-unavailable",
            Self::BuiltinTmuxPrefixStatus => "builtin-tmux-prefix-status", Self::BuiltinTmuxRenameSessionPrompt => "builtin-tmux-rename-session-prompt", Self::BuiltinTmuxRenameWindowPrompt => "builtin-tmux-rename-window-prompt", Self::BuiltinTmuxCommandPrompt => "builtin-tmux-command-prompt", Self::BuiltinTmuxCommandParseError => "builtin-tmux-command-parse-error",
            Self::BuiltinTmuxAttachWindowList => "builtin-tmux-attach-window-list", Self::BuiltinTmuxPaneChooser => "builtin-tmux-pane-chooser", Self::BuiltinTmuxStatusLine => "builtin-tmux-status-line", Self::BuiltinTmuxKillPaneConfirm => "builtin-tmux-kill-pane-confirm", Self::BuiltinTmuxKillWindowConfirm => "builtin-tmux-kill-window-confirm",
            Self::BuiltinTmuxSessionListEntry => "builtin-tmux-session-list-entry",
            Self::BuiltinTmuxClientListEntry => "builtin-tmux-client-list-entry",
            Self::BuiltinTmuxWindowListEntry => "builtin-tmux-window-list-entry",
            Self::BuiltinTmuxPaneListEntry => "builtin-tmux-pane-list-entry",
            Self::BuiltinTmuxPaneNotFound => "builtin-tmux-pane-not-found",
            Self::BuiltinTmuxPaneSizeRequired => "builtin-tmux-pane-size-required", Self::BuiltinTmuxWindowOptionUnsupported => "builtin-tmux-window-option-unsupported", Self::BuiltinTmuxSynchronizePanesRequired => "builtin-tmux-synchronize-panes-required",
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
            Self::BuiltinScreenAttachHelp => "builtin-screen-attach-help", Self::BuiltinScreenWindowListStatus => "builtin-screen-window-list-status",
Self::BuiltinScreenCopyStatus => "builtin-screen-copy-status", Self::BuiltinScreenCopySelectionStatus => "builtin-screen-copy-selection-status",
Self::BuiltinScreenMonitorStatus => "builtin-screen-monitor-status", Self::BuiltinScreenMonitorActivity => "builtin-screen-monitor-activity", Self::BuiltinScreenSilenceStatus => "builtin-screen-silence-status", Self::BuiltinScreenSilenceActivity => "builtin-screen-silence-activity", Self::BuiltinScreenControlMonitorRequired => "builtin-screen-control-monitor-required", Self::BuiltinScreenControlSilenceRequired => "builtin-screen-control-silence-required", Self::BuiltinScreenWrapStatus => "builtin-screen-wrap-status", Self::BuiltinScreenControlWrapRequired => "builtin-screen-control-wrap-required",
            Self::BuiltinScreenAttachHardcopyPathUnavailable => {
                "builtin-screen-attach-hardcopy-path-unavailable"
            }
            Self::BuiltinScreenAttachTitlePrompt => "builtin-screen-attach-title-prompt", Self::BuiltinScreenAttachSelectPrompt => "builtin-screen-attach-select-prompt", Self::BuiltinScreenAttachCommandPrompt => "builtin-screen-attach-command-prompt",
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
            Self::BuiltinScreenControlChdirDirectoryRequired => {
                "builtin-screen-control-chdir-directory-required"
            }
            Self::BuiltinScreenControlChdirHomeRequired => {
                "builtin-screen-control-chdir-home-required"
            }
            Self::BuiltinScreenControlEchoRequired => "builtin-screen-control-echo-required",
            Self::BuiltinScreenControlLastmsgEmpty => "builtin-screen-control-lastmsg-empty",
            Self::BuiltinScreenControlSetenvRequired => "builtin-screen-control-setenv-required",
            Self::BuiltinScreenControlUnsetenvRequired => "builtin-screen-control-unsetenv-required",
            Self::BuiltinScreenControlEnvNameInvalid => "builtin-screen-control-env-name-invalid",
            Self::BuiltinScreenControlShellRequired => "builtin-screen-control-shell-required",
            Self::BuiltinScreenControlShelltitleRequired => "builtin-screen-control-shelltitle-required",
            Self::BuiltinScreenControlTermRequired => "builtin-screen-control-term-required",
            Self::BuiltinScreenControlLogRequired => "builtin-screen-control-log-required",
            Self::BuiltinScreenControlLogtstampRequired => {
                "builtin-screen-control-logtstamp-required"
            }
            Self::BuiltinScreenControlLogfileRequired => {
                "builtin-screen-control-logfile-required"
            }
            Self::BuiltinScreenControlStuffRequired => "builtin-screen-control-stuff-required",
            Self::BuiltinScreenControlRegisterRequired => {
                "builtin-screen-control-register-required"
            },
            Self::BuiltinScreenControlResizeRequired => "builtin-screen-control-resize-required",
            Self::BuiltinScreenControlSizeRequired => "builtin-screen-control-size-required",
            Self::BuiltinScreenControlHelp => "builtin-screen-control-help",
            Self::BuiltinScreenControlSelectUnsupported => "builtin-screen-control-select-unsupported",
            Self::BuiltinScreenControlNumber => "builtin-screen-control-number",
            Self::BuiltinScreenControlNumberInvalid => "builtin-screen-control-number-invalid",
            Self::BuiltinScreenControlScrollbackRequired => {
                "builtin-screen-control-scrollback-required"
            }
            Self::BuiltinScreenControlSleepRequired => "builtin-screen-control-sleep-required",
            Self::BuiltinScreenControlTime => "builtin-screen-control-time",
            Self::BuiltinScreenControlTitleRequired => "builtin-screen-control-title-required",
            Self::BuiltinScreenControlVersion => "builtin-screen-control-version",
            Self::BuiltinScreenControlLicense => "builtin-screen-control-license",
            Self::BuiltinScreenControlInfo => "builtin-screen-control-info",
            Self::BuiltinScreenControlDinfo => "builtin-screen-control-dinfo",
            Self::BuiltinScreenControlDumptermcapComplete => {
                "builtin-screen-control-dumptermcap-complete"
            },
            Self::BuiltinScreenControlDisplaysEntry => "builtin-screen-control-displays-entry",
            Self::BuiltinScreenControlWindowsEntry => "builtin-screen-control-windows-entry",
            Self::BuiltinScreenControlUnexpectedResponse => {
                "builtin-screen-control-unexpected-response"
            }
            Self::BuiltinScreenControlHardcopyPathRequired => {
                "builtin-screen-control-hardcopy-path-required"
            }
            Self::BuiltinScreenControlHardcopydirRequired => {
                "builtin-screen-control-hardcopydir-required"
            }
            Self::BuiltinScreenControlHardcopyAppendRequired => {
                "builtin-screen-control-hardcopy-append-required"
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
            Self::BuiltinScreenControlReadregRequired => {
                "builtin-screen-control-readreg-required"
            }
            Self::BuiltinScreenControlSourcePathRequired => {
                "builtin-screen-control-source-path-required"
            }
            Self::BuiltinScreenControlWritebufPathRequired => {
                "builtin-screen-control-writebuf-path-required"
            }
            Self::BuiltinScreenControlBufferEncodingRequired => {
                "builtin-screen-control-buffer-encoding-required"
            }
            Self::BuiltinScreenControlWritebufComplete => "builtin-screen-control-writebuf-complete", Self::BuiltinScreenControlReadbufComplete => "builtin-screen-control-readbuf-complete", Self::BuiltinScreenControlRemovebufComplete => "builtin-screen-control-removebuf-complete", Self::BuiltinScreenControlBufferIoError => "builtin-screen-control-buffer-io-error",
            Self::BuiltinScreenWipeComplete => "builtin-screen-wipe-complete",
        }
    }
}
