#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TmuxCommand {
    NewSession,
    AttachSession,
    ListSessions,
    KillSession,
    HasSession,
    Other,
}

impl TmuxCommand {
    pub(crate) fn parse(args: &[String]) -> Self {
        match first_command_token(args).map(String::as_str) {
            Some("new" | "new-session") => Self::NewSession,
            Some("attach" | "attach-session") => Self::AttachSession,
            Some("list-sessions" | "ls") => Self::ListSessions,
            Some("kill-session") => Self::KillSession,
            Some("has-session") => Self::HasSession,
            _ => Self::Other,
        }
    }

    pub(crate) fn is_new_session(&self) -> bool {
        matches!(self, Self::NewSession)
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
        assert_eq!(
            TmuxCommand::parse(&["new-session".into()]),
            TmuxCommand::NewSession
        );
        assert_eq!(
            TmuxCommand::parse(&["attach-session".into()]),
            TmuxCommand::AttachSession
        );
        assert_eq!(TmuxCommand::parse(&["ls".into()]), TmuxCommand::ListSessions);
        assert_eq!(
            TmuxCommand::parse(&["kill-session".into()]),
            TmuxCommand::KillSession
        );
        assert_eq!(
            TmuxCommand::parse(&["has-session".into()]),
            TmuxCommand::HasSession
        );
    }

    #[test]
    fn skips_detached_global_flag() {
        assert_eq!(
            TmuxCommand::parse(&["-d".into(), "new".into()]),
            TmuxCommand::NewSession
        );
        assert_eq!(TmuxCommand::parse(&["--detached".into()]), TmuxCommand::Other);
    }
}