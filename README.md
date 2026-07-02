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
terman screen --system --wsl
terman screen --system --no-fallback
# 提示
# - `system` 失败会自动回退到内置 screen；如需强制仅系统模式请加 `--no-fallback`
# - tmux 常见场景与返回码请参考下方「跨平台快速排查（screen / tmux）」
# - Windows 下 tmux 优先使用 `--wsl`
# 使用内置 screen 的登录 shell
terman screen --login-shell
terman screen --help

# 使用 tmux 子命令
terman tmux
terman tmux new -s dev
terman tmux new-session -s dev
terman tmux attach -t <session>
terman tmux attach-session -t <session>
terman tmux list-sessions
terman tmux --detached new -s dev
terman tmux --wsl new -s dev
terman tmux --help

# screen 最小复现示例（可直接复制）

```bash
# 1) system screen 会话不存在（attach）示例
terman screen --system attach -t missing-session
if [ $? -ne 0 ]; then
  echo "screen attach 失败：可先执行 screen --help 检查命令或确认会话是否存在"
fi

# 2) 内置 screen 可复现：启动最小命令
terman screen
```

```powershell
# 1) system screen 会话不存在（attach）示例
terman screen --system attach -t missing-session
if ($LASTEXITCODE -ne 0) {
  Write-Output "screen attach 失败：可先执行 screen --help 检查命令或确认会话是否存在"
}

# 2) 内置 screen 可复现：启动最小命令
terman screen
```

# tmux 最小复现示例（可直接复制）

```bash
# 会话不存在复现
terman tmux attach -t missing-session
if [ $? -ne 0 ]; then
  echo "会话不存在，先执行 list-sessions 确认后重试"
fi

# 会话名冲突复现
terman tmux new -s demo
terman tmux new -s demo
```

```powershell
# 会话不存在复现
terman tmux attach -t missing-session
if ($LASTEXITCODE -ne 0) {
  Write-Output "会话不存在，先执行 list-sessions 确认后重试"
}

# 会话名冲突复现
terman tmux new -s demo
terman tmux new -s demo
```

# Windows 可通过 --wsl 强制使用 WSL tmux；如果 WSL 内 tmux 不可用，会返回安装与环境排查建议.
## 跨平台快速排查（screen / tmux）

### 常见返回码

| 工具 | 典型返回码 | 典型场景 | 排查动作 |
|---|---:|---|---|
| `screen --system` | `1` | 参数错误 / 会话不存在 / 非法操作 | 确认子命令和参数，必要时先用 `terman screen --help` 查看；再用合法参数重试 |
| `screen --system` | `2` | 环境变量或权限问题 | 检查 Shell 环境、WSL 配置和权限，再重试 |
| `screen --system` | `126` | `screen` 可执行文件无法运行（权限/兼容性） | 检查二进制权限与完整安装，或先回退到内置 screen |
| `screen --system` | `127` | `screen` 未找到 | 安装系统 `screen`，或去掉 `--system` 使用内置模式 |
| `tmux`（系统模式） | `1` | 参数错误、会话不存在、attach/list 冲突 | 先执行 `terman tmux list-sessions`，再 `terman tmux attach -t <session>` |
| `tmux`（系统模式） | `2` | 终端环境或权限受限 | 检查 `TERM`、文件系统/权限；Windows 场景优先尝试 `--wsl` |
| `tmux`（系统模式） | `126` | `tmux` 可执行文件无法运行 | 检查 tmux 安装与权限，或尝试 `terman tmux --wsl` |
| `tmux`（系统模式） | `127` | `tmux` 未找到 | 安装 tmux；Windows 下可用 `--wsl`（避免“Windows 原生不可见”） |
| `tmux`（系统模式） | `130` | 用户中断（Ctrl-C） | 按正常退出流程重试 |

### WSL 回退预检说明

- `terman screen --wsl` 会先检查 WSL 内 `screen` 是否可用（等效 `wsl -e which screen`）；
- `terman tmux --wsl` 会先检查 WSL 内 `tmux` 是否可用（等效 `wsl -e which tmux`）；
- 两者都未检测到对应工具时，会在启动前直接返回更明确的排查建议：先执行 `wsl -l -v` 检查发行版、`wsl --status` 检查子系统可用性，再执行 `wsl -e <tool> -V` 验证可用性（screen/tmux 对应 `tool`）；未检测到则提示先安装对应包。

### tmux 失败输出示例

当 `tmux` 执行失败时，当前会输出统一格式：

- `tmux 失败（退出码 <code>）：<场景说明>\n<具体提示>`
- `tmux WSL 预检（退出码 <code>）：<场景说明>\n<安装/排查建议>`
- `tmux 可用性检查（退出码 <code>）：<场景说明>`

例如：

- `terman tmux --wsl attach -t xxx` 若 WSL 下无 tmux：
  - `tmux WSL 预检 失败（退出码 127）：当前已进入 WSL 回退路径...`
- `terman tmux` 在系统 tmux 不可用时：
  - `tmux 失败（退出码 127）：tmux 可用性检查失败...`
### 场景化建议

- `terman tmux attach -t <session>` 报会话不存在：先运行 `terman tmux list-sessions` 确认会话名。
- `terman tmux new -s <name>` 返回会话冲突：改用新的会话名再尝试。
- Windows 下 tmux 启动异常：优先使用 `terman tmux --wsl ...`。
- `terman screen --system attach -t <session>` 报会话不存在：先运行 `terman screen --system -ls` 或 `screen -ls` 确认会话。
- `terman screen --system -S <name>` 遇到会话名冲突：先检查列表并更换会话名。
- `terman screen --system --wsl` 相关异常：按顺序执行 `wsl -l -v`、`wsl --status`、`wsl -e screen -V`，再复现命令。

## 备注

- screen 与 tmux 的 WSL 排查提示已统一复用 `terman-common` 中的共享模板，确保跨平台诊断口径一致。
- 第一目标（跨平台 screen）保持：优先复用成熟工具（`--system`），回退到内置 PTY。
- 第二目标（跨平台 tmux）保持：通过成熟 tmux 工具的托管式桥接执行。

