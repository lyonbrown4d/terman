# terman

一个“现代化跨平台终端工具集合”的起步仓库。

## 当前进展（第一小步）

- 已添加 `terman screen` 命令：基于成熟库 `portable-pty` + `crossterm` + `clap`。
- 它会在当前平台上启动一个 PTY shell，会话中按键通过原始终端事件映射后透传到会话。
- 当终端窗口尺寸变化时，PTY 尺寸会同步更新，避免远端 shell 的显示错位。
- 已增加 `terman tmux` 子命令：优先复用系统可执行的 `tmux`（不重复实现），用于快速接入成熟终端复用体验。
- 这是后续实现真正 screen/tmux 功能（会话列表、断开重连、分屏与窗口布局等）的基础。

## 使用

```bash
# 默认直接进入交互 shell
terman screen

# 运行一段命令（通过当前平台 shell）
terman screen --command "printf \"hello\\n\""

# 直接透传到系统 tmux
terman tmux
terman tmux new -s dev
terman tmux attach -t dev
```

## 备注

- `screen` 当前是可交互的跨平台 PTY 入口，后续会加会话持久化（detached/reconnect）、多窗格等能力。
- `tmux` 当前是“成熟库/工具托管式”实现（先利用系统已有 tmux），在 Windows 上建议使用 WSL/MINGW 生态提供的 tmux。
