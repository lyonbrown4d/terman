# terman

一个“现代化跨平台终端工具集合”的起步仓库。

## 当前进展（第一小步）

- 已添加 `terman screen` 命令：基于成熟库 `portable-pty` + `crossterm` + `clap`。
- 它会在当前平台上启动一个 PTY shell，会话中按键通过原始终端事件映射后透传到会话。
- 这是后续实现真正 screen/tmux 功能（会话列表、断开重连、分屏、窗口布局等）的基础。

## 使用

```bash
# 默认直接进入交互 shell
terman screen

# 运行一段命令（通过当前平台 shell）
terman screen --command "printf "\"hello\\n\""
```

## 备注

- 当前仅是 screen 的第一阶段实现：**可运行的跨平台 shell 会话入口**。
- 下一阶段将补上跨平台会话管理（可 detached/reconnect）、分屏与多窗格，再逐步过渡到 tmux 模块。
