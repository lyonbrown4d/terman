# terman

一个“现代化跨平台终端工具集合”的起步仓库。

## 当前进展（第三阶段）

- 已添加 `terman screen` 命令：基于成熟库 `portable-pty` + `crossterm` + `clap`。
- 实现跨平台 PTY 输入输出，支持窗口大小变化同步。
- 已添加 `terman tmux` 命令：默认优先复用系统 `tmux`，Windows 下自动尝试 `wsl tmux` 兜底，不做自建实现。
- 这是“先把成熟工具接入再逐步演进”的第一条完整路径。

## 使用

```bash
# 默认直接进入交互 shell（内置 screen 交互入口）
terman screen

# 运行一段命令（通过当前平台 shell）
terman screen --command "printf \"hello\\n\""

# 使用系统 tmux（建议安装后直接使用）
terman tmux
terman tmux new -s dev
terman tmux attach -t dev
```

## 备注

- `screen` 当前仍是第一阶段：可交互 PTY 会话。
- `tmux` 当前是托管式桥接：尽量调用本机成熟实现，Windows 先尝试 WSL 方案。
