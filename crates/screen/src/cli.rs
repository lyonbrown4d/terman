use clap::{Args, Parser};

#[derive(Args, Debug, Clone)]
#[command(
    about = "screen 桥接入口（先尝试系统 screen，失败自动回退内置）",
    after_help = "常见用法示例：\n  - terman-screen\n  - terman-screen -S dev\n  - terman-screen --list\n  - terman-screen -r dev\n  - terman-screen -x dev\n  - terman-screen --system\n  - terman-screen --system --list\n  - terman-screen --system -r dev\n  - terman-screen --system -S dev\n  - terman-screen --system --detach\n  - terman-screen --system --no-fallback"
)]
pub struct ScreenArgs {
    /// If set, run this command string through the platform shell in built-in mode.
    #[arg(short, long, value_name = "CMD", conflicts_with = "system")]
    pub command: Option<String>,

    /// Initial terminal columns.
    #[arg(long)]
    pub cols: Option<u16>,

    /// Initial terminal rows.
    #[arg(long)]
    pub rows: Option<u16>,

    /// Name the screen session; maps to `screen -S <NAME>` in system mode.
    #[arg(short = 'S', long = "session", value_name = "NAME")]
    pub session_name: Option<String>,

    /// List known screen sessions. In system mode this maps to `screen -ls`.
    #[arg(long, alias = "ls", conflicts_with_all = ["command", "detach"])]
    pub list: bool,

    /// Resume a detached screen session; maps to `screen -r [NAME]` in system mode.
    #[arg(
        short = 'r',
        long = "resume",
        value_name = "NAME",
        num_args = 0..=1,
        conflicts_with_all = ["command", "detach", "list", "session_name", "multi_attach"]
    )]
    pub resume: Option<Option<String>>,

    /// Attach to an existing session without detaching other displays; maps to `screen -x [NAME]`.
    #[arg(
        short = 'x',
        long = "multi-attach",
        value_name = "NAME",
        num_args = 0..=1,
        conflicts_with_all = ["command", "detach", "list", "session_name", "resume"]
    )]
    pub multi_attach: Option<Option<String>>,

    /// Prefer using system `screen` if available.
    #[arg(long)]
    pub system: bool,

    /// 启动系统 screen 后台模式（等价于 `screen -d -m`）。
    #[arg(long, requires = "system")]
    pub detach: bool,

    /// Start a login shell in built-in mode; ignored in system mode.
    #[arg(long, conflicts_with = "system")]
    pub login_shell: bool,

    /// 回退到内置 screen（`--system` 启动失败时）。
    #[arg(long)]
    pub no_fallback: bool,

    /// Extra args passed to system screen when `--system` is enabled.
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}

impl Default for ScreenArgs {
    fn default() -> Self {
        Self {
            command: None,
            cols: None,
            rows: None,
            session_name: None,
            list: false,
            resume: None,
            multi_attach: None,
            system: false,
            detach: false,
            login_shell: false,
            no_fallback: false,
            args: Vec::new(),
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