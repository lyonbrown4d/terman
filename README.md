# terman

一个现代化跨平台终端工具集合的 Rust monorepo。这里的集合指仓库管理方式，不是单个主命令分发器：screen 就是 screen，tmux 就是 tmux，它们是互不关联的独立命令行终端工具。

## 当前结构

- `crates/screen`：生成 `terman-screen`，承载跨平台 screen 兼容能力和 PTY 会话运行时。
- `crates/tmux`：生成 `terman-tmux`，承载跨平台 tmux 兼容命令和会话运行时。
- `crates/common`：共享 i18n 提示、跨平台路径和命令行工具公共能力。
- 根目录是 Cargo virtual workspace，不再提供 `terman` 主入口。

## 构建

```bash
cargo build --workspace
```

构建后可直接运行：

```bash
./target/debug/terman-screen --help
./target/debug/terman-tmux --help
```

Windows PowerShell：

```powershell
.\target\debug\terman-screen.exe --help
.\target\debug\terman-tmux.exe --help
```

## screen 使用

```bash
# 启动跨平台 screen 会话
terman-screen

# 创建命名会话
terman-screen -S dev

# 连接已有会话
terman-screen -r dev

# 运行一个命令
terman-screen --command "printf \"hello\\n\""

# 使用登录 shell
terman-screen --login-shell

# 向会话发送 screen -X 控制命令
terman-screen -S dev -X windows
terman-screen -S dev -X screen
terman-screen -S dev -X select 0
```

screen 最小复现示例：

```bash
terman-screen -r missing-session
if [ $? -ne 0 ]; then
  echo "screen 会话不存在：可先执行 terman-screen --list 检查会话"
fi
```

```powershell
terman-screen -r missing-session
if ($LASTEXITCODE -ne 0) {
  Write-Output "screen 会话不存在：可先执行 terman-screen --list 检查会话"
}
```

## tmux 使用

```bash
terman-tmux
terman-tmux new -s dev
terman-tmux new-session -s dev
terman-tmux attach -t <session>
terman-tmux attach-session -t <session>
terman-tmux list-sessions
terman-tmux list-windows -t dev
terman-tmux split-window -t dev
terman-tmux split-window -h -t dev "cargo run"
terman-tmux list-panes -t dev
terman-tmux select-pane -t dev:0.1
terman-tmux --detached new -s dev
```

tmux 最小复现示例：

```bash
terman-tmux attach -t missing-session
if [ $? -ne 0 ]; then
  echo "会话不存在，先执行 list-sessions 确认后重试"
fi

terman-tmux new -s demo
terman-tmux list-sessions
```

```powershell
terman-tmux attach -t missing-session
if ($LASTEXITCODE -ne 0) {
  Write-Output "会话不存在，先执行 list-sessions 确认后重试"
}

terman-tmux new -s demo
terman-tmux list-sessions
```

Windows 就走 Windows 本机能力，Linux 就走 Linux 本机能力；项目不引入 WSL、系统命令代理或跨系统兼容后端。

## 跨平台快速排查

| 工具 | 典型返回码 | 典型场景 | 排查动作 |
|---|---:|---|---|
| `terman-screen` | `1` | 参数错误 / 会话不存在 / 非法操作 | 确认子命令和参数，必要时先用 `terman-screen --help` 查看；再用合法参数重试 |
| `terman-screen` | `2` | 终端环境或权限受限 | 检查 Shell 环境、`TERM`、文件系统和会话目录权限 |
| `terman-screen` | `130` | 用户中断（Ctrl-C） | 按正常退出流程重试 |
| `terman-tmux` | `1` | 参数错误、会话不存在、attach/list 冲突 | 先执行 `terman-tmux list-sessions`，再 `terman-tmux attach -t <session>` |
| `terman-tmux` | `2` | 终端环境或权限受限 | 检查 `TERM`、文件系统权限和会话目录权限 |
| `terman-tmux` | `130` | 用户中断（Ctrl-C） | 按正常退出流程重试 |

## 备注

- 第一目标：`terman-screen` 自身实现跨平台 screen 能力，不桥接系统 `screen`。
- 第二目标：`terman-tmux` 自身实现跨平台 tmux 能力，不桥接系统 `tmux`。
- 跨平台提示复用 `terman-common` 中打包进二进制的 i18n 资源，确保 screen/tmux 的诊断口径一致。
