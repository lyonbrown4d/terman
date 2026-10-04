# terman

`terman` 是一个基于 Go Workspace 管理的现代跨平台终端工具 monorepo。仓库不提供统一主入口，`terman-screen`、`terman-tmux` 和 `terman-htop` 是互不关联的独立进程与命令。

## Workspace

```text
terman/
├── go.work
├── common/   # 共享终端、shell、命令执行和嵌入式 i18n
├── screen/   # 独立 screen 实现
├── tmux/     # 独立 tmux 实现
└── htop/     # 独立 htop 实现
```

每个目录拥有独立 `go.mod`。根 `go.work` 只负责本地协作和统一构建，不引入总命令或单进程分发器。

## 平台原则

- Windows 使用 Windows 原生 ConPTY 和 Named Pipe。
- Linux 使用 Linux PTY 和 Unix Domain Socket。
- 不调用 WSL、系统 `screen` 或系统 `tmux` 作为兼容后端。
- 本地化资源通过 `embed` 打包进二进制。

## 主要依赖

- `github.com/aymanbagabas/go-pty`：Unix PTY 与 Windows ConPTY。
- `github.com/charmbracelet/x/vt`：VT/xterm 状态解析。
- `github.com/gdamore/tcell`：全屏终端、键盘和鼠标事件。
- `github.com/Microsoft/go-winio`：Windows Named Pipe IPC。
- `github.com/shirou/gopsutil/v4`：跨平台系统与进程指标。

ArcGoLabs 生态优先用于适配的外围基础设施；当前公开仓库未声明许可证的组件不会直接进入依赖树。

## 构建

在仓库根目录执行：

```bash
go build ./common/... ./screen/... ./tmux/... ./htop/...
```

构建独立命令：

```bash
go build -o bin/terman-screen ./screen/cmd/terman-screen
go build -o bin/terman-tmux ./tmux/cmd/terman-tmux
go build -o bin/terman-htop ./htop/cmd/terman-htop
```

Windows PowerShell 可将输出名改为 `.exe`。

## 使用

```bash
terman-screen --help
terman-screen -S dev
terman-screen -r dev
terman-screen --list

terman-tmux --help
terman-tmux new -s dev
terman-tmux attach -t dev
terman-tmux list-sessions

terman-htop --help
terman-htop --refresh-ms 1000
terman-htop --once
```

## 终端交互

- screen 前缀：`Ctrl-A`。
- tmux 前缀：`Ctrl-B`。
- htop：Tab/Shift-Tab 或方向键切换视图，支持鼠标，`Ctrl-C`、`q`、`F10` 退出。