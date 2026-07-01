# terman

一个“现代化跨平台终端工具集合”的起步仓库。

## 当前进展（第四阶段）

- 已添加 `terman screen` 命令：基于成熟库 `portable-pty` + `crossterm` + `clap`。
- 实现跨平台 PTY 输入输出，支持窗口大小变化同步。
- 已添加 `terman tmux` 命令：默认优先复用系统 `tmux`，Windows 下自动尝试 `wsl tmux` 兜底，不做自建实现。
- `screen` 命令新增 `--system`：优先复用系统 `screen`（若存在），否则回退到 `terman` 内置 PTY 模式，形成“成熟工具优先，回退兜底”的屏幕会话路径。
- `screen` 与 `tmux` 目前作为同一进程入口的两个子命令，后续可无缝迁移到 monorepo 的多包结构。

## 使用

```bash
# 默认直接进入交互 shell（内置 screen 交互入口）
terman screen

# 内置 screen 运行命令
terman screen --command "printf \"hello\\n\""

# 使用系统 screen（若已安装）
terman screen --system
terman screen --system -S dev

# 使用系统 tmux（建议安装后直接使用）
terman tmux
terman tmux new -s dev
terman tmux attach -t dev
```

## 备注

- `screen` 当前仍是第一阶段：可交互 PTY 会话，逐步推进到会话管理、断开重连与多窗格。
- `tmux` 当前是托管式桥接：尽量调用本机成熟实现，Windows 先尝试 WSL 方案。
