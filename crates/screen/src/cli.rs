use clap::{Args, Parser};

#[derive(Args, Debug, Clone)]
#[command(
    about = "跨平台 screen 终端会话工具（自实现内置后端）",
    after_help = "常见用法示例：\n  - terman-screen\n  - terman-screen -S dev\n  - terman-screen --list\n  - terman-screen -S dev -X quit\n  - terman-screen -S dev -X stuff \"echo hi\\n\"\n  - terman-screen -r dev\n  - terman-screen -x dev"
)]
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

    /// List known screen sessions.
    #[arg(long, alias = "ls", conflicts_with = "command")]
    pub list: bool,

    /// Execute a control command against an existing screen session.
    #[arg(
        short = 'X',
        long = "execute",
        value_name = "COMMAND",
        conflicts_with_all = ["command", "list", "resume", "multi_attach", "internal_server"]
    )]
    pub execute: Option<String>,

    /// Extra arguments for the screen control command.
    #[arg(value_name = "ARG", trailing_var_arg = true, requires = "execute")]
    pub execute_args: Vec<String>,

    /// Resume a detached screen session once the built-in session service is available.
    #[arg(
        short = 'r',
        long = "resume",
        value_name = "NAME",
        num_args = 0..=1,
        conflicts_with_all = ["command", "list", "session_name", "multi_attach", "execute"]
    )]
    pub resume: Option<Option<String>>,

    /// Attach to an existing session without detaching other displays once the built-in session service is available.
    #[arg(
        short = 'x',
        long = "multi-attach",
        value_name = "NAME",
        num_args = 0..=1,
        conflicts_with_all = ["command", "list", "session_name", "resume", "execute"]
    )]
    pub multi_attach: Option<Option<String>>,

    /// Start a login shell when supported by the platform shell.
    #[arg(long)]
    pub login_shell: bool,

    /// Internal headless session server mode.
    #[arg(long = "__screen-server", hide = true)]
    pub internal_server: bool,
}

impl Default for ScreenArgs {
    fn default() -> Self {
        Self {
            command: None,
            cols: None,
            rows: None,
            session_name: None,
            list: false,
            execute: None,
            execute_args: Vec::new(),
            resume: None,
            multi_attach: None,
            login_shell: false,
            internal_server: false,
        }
    }
}

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    args: ScreenArgs,
}

pub fn run_with_binary_parse() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    crate::run(cli.args)
}