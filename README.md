# terman

一个“现代化跨平台终端工具集合”的起步仓库。

## 当前进展（第五阶段）：Monorepo 骨架

- `terman` 为主 CLI 分发入口：按子命令转发到独立二进制。
- 引入 workspace 成员库：
  - `crates/screen`：承载 screen 相关能力（内置 PTY + 可选 system screen 委托）。
  - `crates/tmux`：承载 tmux 委托能力（Windows 优先 WSL 回退）。
- 子命令是独立进程：`terman-screen`、`terman-tmux`，`terman` 默认执行 screen。

## 使用

```bash
# 默认进入 screen 模式
terman

# 显式调用 screen 子命令
terman screen
terman screen --command "printf \"hello\\n\""

# 使用系统 screen（若已安装）
terman screen --system
terman screen --system -S dev
terman screen --system --detach
terman screen --system --no-fallback
# 系统 screen 常见返回码
#   1: 参数错误或会话不存在
#   2: 环境变量/权限类异常
#   126: executable 无法执行
#   127: executable 未找到
# 默认行为：system 失败会自动回退到内置 screen；如需禁用请加 --no-fallback
# tmux 常见返回码（系统模式桥接）
#   1: 参数错误、会话不存在、或 tmux 当前状态不允许该操作
#   2: 执行受限（权限或文件系统/终端环境）
#   126: executable 无法执行（如权限不足）
#   127: executable 未找到（Windows 建议优先尝试 `--wsl`）
#   130: 用户中断（Ctrl-C）
# tmux 排查建议
#   发现 attach 报错且不确定会话：terman tmux list-sessions
#   明确指定会话再重试：terman tmux attach -t <session-name>
#   仅 Windows 需要回退到 WSL 时加 --wsl：terman tmux --wsl new -s <session>
# 使用内置 screen 的登录 shell
terman screen --login-shell
terman screen --help

# 使用 tmux 子命令
terman tmux
terman tmux new -s dev
terman tmux attach -t dev
terman tmux --detached
terman tmux --detached new -s dev
terman tmux --wsl new -s dev
terman tmux list-sessions
terman tmux attach -t <session-name>
terman tmux --help

# Windows 可通过 --wsl 强制使用 WSL tmux
tmux 命令若不可用会给出安装路径提示（如 WSL/system 识别与安装建议）
失败时会给出常见场景建议（如 attach 未显式指定会话）。
```

## 备注

- 第一目标（跨平台 screen）保持：优先复用成熟工具（`--system`），回退到内置 PTY。
- 第二目标（跨平台 tmux）保持：通过成熟 tmux 工具的托管式桥接执行。

