# terman

一个“现代化跨平台终端工具集合”的起步仓库。

## 当前进展（第五阶段）：Monorepo 骨架

- `terman` 保持主 CLI 入口（`src/main.rs`）。
- 引入 workspace 成员库：
  - `crates/screen`：承载 screen 相关能力（内置 PTY + 可选 system screen 委托）。
  - `crates/tmux`：承载 tmux 委托能力（Windows 优先 WSL 回退）。
- 这是 `screen` 与 `tmux` 两条能力线的第一层结构化拆分，后续可继续把每一层做成独立 crate 发布。

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

- 第一目标（跨平台 screen）保持：优先复用成熟工具（`--system`），退化到内置 PTY 会话。
- 第二目标（跨平台 tmux）保持：通过成熟 tmux 工具的托管式桥接执行。
- 下一步建议先做 `crates/screen` 的 API 模块化（会话枚举、detached/attach 命令转换），再在 `crates/tmux` 做相同策略。
