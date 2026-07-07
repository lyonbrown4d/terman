use std::error::Error;

use clap::{Args, CommandFactory, FromArgMatches, Parser};

#[derive(Clone, Args, Debug)]
pub struct TmuxArgs {
    /// 等价于 tmux -d，启动会话前台/后台分离。
    /// 已开启 `--detached` 且未显式使用 `new/new-session` 时，tmux 可能按默认行为忽略或返回不同结果。
    #[arg(long)]
    pub detached: bool,

    /// Internal headless session server mode.
    #[arg(long = "__tmux-server", hide = true)]
    pub internal_server: bool,

    /// Internal server session name.
    #[arg(long = "__session-name", hide = true, requires = "internal_server")]
    pub internal_session_name: Option<String>,

    /// Internal server IPC endpoint name.
    #[arg(long = "__endpoint-name", hide = true, requires = "internal_server")]
    pub internal_endpoint_name: Option<String>,

    /// Arguments parsed by the built-in tmux-compatible command dispatcher.
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    args: TmuxArgs,
}

pub fn run_with_binary_parse() -> Result<(), Box<dyn Error>> {
    let matches = Cli::command()
        .about(terman_common::builtin_tmux_cli_about())
        .after_help(terman_common::builtin_tmux_cli_examples())
        .try_get_matches()?;
    let cli = Cli::from_arg_matches(&matches)?;
    crate::run(cli.args)
}
