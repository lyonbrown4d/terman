use std::error::Error;

use clap::{Args, Parser};

#[derive(Args, Debug)]
#[command(
    about = "tmux 桥接入口（按当前平台的原生命令参数透传）",
    after_help = "常见用法示例：\n  - terman-tmux new -s dev\n  - terman-tmux new-session -s dev\n  - terman-tmux attach -t <session>\n  - terman-tmux attach-session -t <session>\n  - terman-tmux list-sessions\n  - terman-tmux --detached new -s dev\n\n排查示例（最小复现）：\n  - 会话不存在：terman-tmux attach -t missing-session\n  - 先查看会话：terman-tmux list-sessions\n  - 名称冲突：terman-tmux new -s demo\n  - 再复现冲突：terman-tmux new -s demo\n"
)]
pub struct TmuxArgs {
    /// 等价于 tmux -d，启动会话前台/后台分离。
    /// 已开启 `--detached` 且未显式使用 `new/new-session` 时，tmux 可能按默认行为忽略或返回不同结果。
    #[arg(long)]
    pub detached: bool,

    /// Directly passed arguments for tmux.
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    args: TmuxArgs,
}

pub fn run_with_binary_parse() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    crate::run(cli.args)
}