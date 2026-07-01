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

# 使用内置 screen 的登录 shell
terman screen --login-shell

# 使用 tmux 子命令
terman tmux
terman tmux new -s dev
terman tmux attach -t dev
terman tmux --detached
terman tmux --detached new -s dev
```

## 备注

- 第一目标（跨平台 screen）保持：优先复用成熟工具（`--system`），回退到内置 PTY。
- 第二目标（跨平台 tmux）保持：通过成熟 tmux 工具的托管式桥接执行。