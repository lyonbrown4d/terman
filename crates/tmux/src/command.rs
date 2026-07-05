#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TmuxCommand {
    NewSession,
    AttachSession,
    ListSessions,
    KillSession,
    KillServer,
    HasSession,
    RenameSession,
    DisplayMessage,
    NewWindow,
    ListWindows,
    KillWindow,
    RenameWindow,
    Other,
}

impl TmuxCommand {
    pub(crate) fn parse(args: &[String]) -> Self {
        match first_command_token(args).map(String::as_str) {
            Some("new" | "new-session") => Self::NewSession,
            Some("attach" | "attach-session") => Self::AttachSession,
            Some("list-sessions" | "ls") => Self::ListSessions,
            Some("kill-session") => Self::KillSession,
            Some("kill-server") => Self::KillServer,
            Some("has-session") => Self::HasSession,
            Some("rename-session") => Self::RenameSession,
            Some("display-message" | "display") => Self::DisplayMessage,
            Some("new-window" | "neww") => Self::NewWindow,
            Some("list-windows" | "lsw") => Self::ListWindows,
            Some("kill-window" | "killw") => Self::KillWindow,
            Some("rename-window" | "renamew") => Self::RenameWindow,
            _ => Self::Other,
        }
    }
}

fn first_command_token(args: &[String]) -> Option<&String> {
    args.iter().find(|arg| !is_global_option_or_value(arg))
}

fn is_global_option_or_value(arg: &str) -> bool {
    matches!(arg, "-d" | "--detached")
}

#[cfg(test)]
mod tests {
    use super::TmuxCommand;

    #[test]
    fn parses_tmux_command_aliases() {
        assert_eq!(TmuxCommand::parse(&["new".into()]), TmuxCommand::NewSession);
        assert_eq!(TmuxCommand::parse(&["new-session".into()]), TmuxCommand::NewSession);
        assert_eq!(TmuxCommand::parse(&["attach-session".into()]), TmuxCommand::AttachSession);
        assert_eq!(TmuxCommand::parse(&["ls".into()]), TmuxCommand::ListSessions);
        assert_eq!(TmuxCommand::parse(&["kill-session".into()]), TmuxCommand::KillSession);
        assert_eq!(TmuxCommand::parse(&["kill-server".into()]), TmuxCommand::KillServer);
        assert_eq!(TmuxCommand::parse(&["has-session".into()]), TmuxCommand::HasSession);
        assert_eq!(TmuxCommand::parse(&["rename-session".into()]), TmuxCommand::RenameSession);
        assert_eq!(TmuxCommand::parse(&["display".into()]), TmuxCommand::DisplayMessage);
        assert_eq!(TmuxCommand::parse(&["neww".into()]), TmuxCommand::NewWindow);
        assert_eq!(TmuxCommand::parse(&["lsw".into()]), TmuxCommand::ListWindows);
        assert_eq!(TmuxCommand::parse(&["killw".into()]), TmuxCommand::KillWindow);
        assert_eq!(TmuxCommand::parse(&["renamew".into()]), TmuxCommand::RenameWindow);
    }

    #[test]
    fn skips_detached_global_flag() {
        assert_eq!(TmuxCommand::parse(&["-d".into(), "new".into()]), TmuxCommand::NewSession);
        assert_eq!(TmuxCommand::parse(&["--detached".into()]), TmuxCommand::Other);
    }
}
