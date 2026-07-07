use std::ffi::OsString;

use clap::{Args, CommandFactory, FromArgMatches, Parser};

#[derive(Args, Debug, Clone)]
pub struct ScreenArgs {
    /// If set, run this command string through the platform shell in built-in mode.
    #[arg(short, long, value_name = "CMD")]
    pub command: Option<String>,

    /// Initial terminal columns.
    #[arg(long)]
    pub cols: Option<u16>,

    /// Initial terminal rows.
    #[arg(long)]
    pub rows: Option<u16>,

    /// Name the screen session.
    #[arg(short = 'S', long = "session", value_name = "NAME")]
    pub session_name: Option<String>,

    /// Start a named screen session without attaching to it.
    #[arg(
        short = 'd',
        long = "detached",
        requires = "session_name",
        conflicts_with_all = ["list", "wipe", "resume", "multi_attach", "execute", "internal_server"]
    )]
    pub detached: bool,

    /// Accept GNU screen detach-existing attach syntax; attach behavior is handled by the built-in session service.
    #[arg(
        long = "detach-existing",
        hide = true,
        conflicts_with_all = ["command", "list", "wipe", "execute", "internal_server"]
    )]
    pub detach_existing: bool,

    /// List known screen sessions.
    #[arg(long, alias = "ls", conflicts_with_all = ["command", "wipe"])]
    pub list: bool,

    /// Emit known screen sessions as JSON. Requires --list.
    #[arg(long, requires = "list")]
    pub json: bool,

    /// Remove stale screen session records.
    #[arg(
        long,
        conflicts_with_all = ["command", "list", "resume", "multi_attach", "execute", "internal_server"]
    )]
    pub wipe: bool,

    /// Select a screen window for a control command.
    #[arg(
        short = 'p',
        long = "window",
        value_name = "WINDOW",
        requires = "execute",
        conflicts_with_all = ["command", "list", "wipe", "resume", "multi_attach", "internal_server"]
    )]
    pub window_selector: Option<String>,

    /// Execute or query a control command against an existing screen session.
    #[arg(
        short = 'X',
        short_alias = 'Q',
        long = "execute",
        alias = "query",
        value_name = "COMMAND",
        conflicts_with_all = ["command", "list", "wipe", "resume", "multi_attach", "internal_server"]
    )]
    pub execute: Option<String>,

    /// Extra arguments for the screen control command.
    #[arg(value_name = "ARG", trailing_var_arg = true, allow_hyphen_values = true, requires = "execute")]
    pub execute_args: Vec<String>,

    /// Resume an existing session, or create a named session when it does not exist.
    #[arg(
        short = 'R',
        long = "resume-or-create",
        value_name = "NAME",
        conflicts_with_all = [
            "command",
            "list",
            "wipe",
            "session_name",
            "resume",
            "multi_attach",
            "execute",
            "internal_server"
        ]
    )]
    pub resume_or_create: Option<String>,

    /// Resume a detached screen session once the built-in session service is available.
    #[arg(
        short = 'r',
        long = "resume",
        value_name = "NAME",
        num_args = 0..=1,
        conflicts_with_all = ["command", "list", "wipe", "session_name", "multi_attach", "execute"]
    )]
    pub resume: Option<Option<String>>,

    /// Attach to an existing session without detaching other displays once the built-in session service is available.
    #[arg(
        short = 'x',
        long = "multi-attach",
        value_name = "NAME",
        num_args = 0..=1,
        conflicts_with_all = ["command", "list", "wipe", "session_name", "resume", "execute"]
    )]
    pub multi_attach: Option<Option<String>>,

    /// Start a login shell when supported by the platform shell.
    #[arg(long)]
    pub login_shell: bool,

    /// Internal headless session server mode.
    #[arg(long = "__screen-server", hide = true)]
    pub internal_server: bool,

    /// Internal server IPC endpoint name.
    #[arg(long = "__endpoint-name", hide = true, requires = "internal_server")]
    pub internal_endpoint_name: Option<String>,
}

impl Default for ScreenArgs {
    fn default() -> Self {
        Self {
            command: None,
            cols: None,
            rows: None,
            session_name: None,
            detached: false,
            detach_existing: false,
            list: false,
            json: false,
            wipe: false,
            window_selector: None,
            execute: None,
            execute_args: Vec::new(),
            resume_or_create: None,
            resume: None,
            multi_attach: None,
            login_shell: false,
            internal_server: false,
            internal_endpoint_name: None,
        }
    }
}

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    args: ScreenArgs,
}

pub fn run_with_binary_parse() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Cli::command()
        .about(terman_common::builtin_screen_cli_about())
        .after_help(terman_common::builtin_screen_cli_examples())
        .try_get_matches_from(normalize_screen_args(std::env::args_os()))?;
    let cli = Cli::from_arg_matches(&matches)?;
    crate::run(cli.args)
}

fn normalize_screen_args(args: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    let args: Vec<OsString> = args.into_iter().collect();
    let attach_requested = args.iter().any(is_attach_arg);
    let mut normalized = Vec::new();

    for arg in args {
        match arg.to_str() {
            Some("-ls") | Some("-list") => normalized.push(OsString::from("--list")),
            Some("-wipe") => normalized.push(OsString::from("--wipe")),
            Some("-D") => normalized.push(OsString::from("--detach-existing")),
            Some("-d") if attach_requested => normalized.push(OsString::from("--detach-existing")),
            Some("-m") => {}
            Some("-dm") => normalized.push(OsString::from("-d")),
            Some("-dmS") => {
                normalized.push(OsString::from("-d"));
                normalized.push(OsString::from("-S"));
            }
            Some(value) if value.starts_with("-dmS") && value.len() > 4 => {
                normalized.push(OsString::from("-d"));
                normalized.push(OsString::from("-S"));
                normalized.push(OsString::from(&value[4..]));
            }
            _ => normalized.push(arg),
        }
    }

    normalized
}

fn is_attach_arg(arg: &OsString) -> bool {
    matches!(
        arg.to_str(),
        Some("-r") | Some("--resume") | Some("-R") | Some("--resume-or-create") | Some("-x") | Some("--multi-attach")
    )
}

