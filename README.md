# terman

一个现代化跨平台终端工具集合的 monorepo。这里的集合指仓库管理方式，不是单个主命令分发器：screen 就是 screen，tmux 就是 tmux，它们是互不关联的独立命令行终端工具。

## 当前结构

- `crates/screen`：生成 `terman-screen`，承载 screen 相关能力（内置 PTY + 可选 system screen 委托）。
- `crates/tmux`：生成 `terman-tmux`，承载当前平台的 tmux 委托能力。
- `crates/common`：共享跨平台检测、i18n 提示和环境变量透传工具。
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
# 启动内置 screen 模式
terman-screen

# 运行一个命令
terman-screen --command "printf \"hello\\n\""

# 使用系统 screen
terman-screen --system
terman-screen --system -S dev
terman-screen --system --detach
terman-screen --system --no-fallback
terman-screen --system --no-fallback

# 使用内置 screen 的登录 shell
terman-screen --login-shell
```

screen 最小复现示例：

```bash
terman-screen --system -r missing-session
if [ $? -ne 0 ]; then
  echo "screen 会话不存在：可先执行 screen -ls 检查命令或确认会话是否存在"
fi
```

```powershell
terman-screen --system -r missing-session
if ($LASTEXITCODE -ne 0) {
  Write-Output "screen 会话不存在：可先执行 screen -ls 检查命令或确认会话是否存在"
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
terman-tmux --detached new -s dev
terman-tmux --detached new -s dev
```

tmux 最小复现示例：

```bash
terman-tmux attach -t missing-session
if [ $? -ne 0 ]; then
  echo "会话不存在，先执行 list-sessions 确认后重试"
fi

terman-tmux new -s demo
terman-tmux new -s demo
```

```powershell
terman-tmux attach -t missing-session
if ($LASTEXITCODE -ne 0) {
  Write-Output "会话不存在，先执行 list-sessions 确认后重试"
}

terman-tmux new -s demo
terman-tmux new -s demo
```

Windows 就走 Windows 本机能力，Linux 就走 Linux 本机能力；项目不把 WSL 作为运行后端或依赖路径。

## 跨平台快速排查

| 工具 | 典型返回码 | 典型场景 | 排查动作 |
|---|---:|---|---|
| `terman-screen --system` | `1` | 参数错误 / 会话不存在 / 非法操作 | 确认子命令和参数，必要时先用 `terman-screen --help` 查看；再用合法参数重试 |
| `terman-screen --system` | `2` | 环境变量或权限问题 | 检查 Shell 环境、本机可执行文件和权限，再重试 |
| `terman-screen --system` | `126` | `screen` 可执行文件无法运行 | 检查二进制权限与完整安装，或先回退到内置 screen |
| `terman-screen --system` | `127` | `screen` 未找到 | 安装系统 `screen`，或去掉 `--system` 使用内置模式 |
| `terman-tmux` | `1` | 参数错误、会话不存在、attach/list 冲突 | 先执行 `terman-tmux list-sessions`，再 `terman-tmux attach -t <session>` |
| `terman-tmux` | `2` | 终端环境或权限受限 | 检查 `TERM`、文件系统/权限和本机 tmux 路径 |
| `terman-tmux` | `126` | `tmux` 可执行文件无法运行 | 检查本机 tmux 安装与权限 |
| `terman-tmux` | `127` | `tmux` 未找到 | 安装当前平台的 tmux 可执行文件 |
| `terman-tmux` | `130` | 用户中断（Ctrl-C） | 按正常退出流程重试 |

## 备注

- 第一目标（跨平台 screen）保持：优先复用成熟工具（`--system`），回退到内置 PTY。
- 第二目标（跨平台 tmux）保持：通过成熟 tmux 工具的托管式桥接执行。
- 跨平台提示复用 `terman-common` 中打包进二进制的 i18n 资源，确保 screen/tmux 的诊断口径一致。
