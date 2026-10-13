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

## 安装包与发布

GoReleaser 开源版在根目录统一构建三个独立命令，保留 `go.work` 的本地模块解析。
发布使用统一的 `vX.Y.Z` 标签；预发布标签（例如 `v0.3.0-rc.1`）自动标记为预发布。

| 平台 | 架构 | 每个工具的安装包 |
| --- | --- | --- |
| Windows | amd64、arm64 | ZIP，解压后将可执行文件所在目录加入 PATH |
| Linux | amd64、arm64 | tar.gz、DEB、RPM；DEB/RPM 安装到 `/usr/bin` |

文件名包含工具名、版本、平台和架构。每个版本同时提供 SHA-256 校验文件。
除各工具的独立包外，`terman_VERSION_OS_ARCH.zip` / `.tar.gz` 包含三个独立可执行文件。
三个工具都支持 `--version`，正式安装包显示标签对应的版本号，普通本地构建显示 `dev`。
本地化资源仍嵌入二进制，无需额外下载资源文件。

所有发布二进制通过 UPX 5.2.1 或更新稳定版的 `--best` 压缩，随后运行 `upx --test`。
压缩使用 GoReleaser 原生 `upx:` 配置；发布前必须通过 snapshot 的全部二进制完整性及镜像运行验证。
Windows ARM64 使用 NRV 压缩，不传入不受支持的 LZMA 参数。
发布 CI 在 Windows/Linux 的 amd64/arm64 原生 runner 上运行测试，并用压缩后的二进制执行真实 Screen PTY E2E。

从 [GitHub Releases](https://github.com/lyonbrown4d/terman/releases) 下载对应安装包。
Linux 可以使用 `sudo apt install ./terman-screen_*.deb` 或
`sudo rpm -Uvh terman-screen-*.rpm` 安装；其他工具安装方式相同。

本地需安装 Go、GoReleaser、UPX 和支持 buildx 的 Docker。
试打包使用全局 Go 缓存，产物统一写入仓库外的 `../terman-release`：

```powershell
goreleaser check
goreleaser release --snapshot --clean
```

Linux 示例：

```bash
goreleaser check
goreleaser release --snapshot --clean
```

`--clean` 会清空 `../terman-release`，请为它保留专用目录。
只验证二进制与安装包时可加 `--skip=docker`。
推送 `v*` 标签时上传到 GitHub Releases，并将容器镜像发布到 GHCR；PR 和手动执行生成 snapshot。
标签发布也先生成 snapshot，验证 12 个压缩二进制和两个架构的容器功能，再执行正式发布。

## Docker 镜像

三个镜像分别为 `ghcr.io/lyonbrown4d/terman-screen`、`ghcr.io/lyonbrown4d/terman-tmux`、
`ghcr.io/lyonbrown4d/terman-htop`，每个支持 `linux/amd64` 和 `linux/arm64`。
版本标签为 `vX.Y.Z`；仅正式版本更新 `latest`，预发布不覆盖它。
镜像直接打包同一批 UPX 二进制，包含原生 shell 和终端数据，默认以 UID/GID 10001 运行。
发布使用仓库的 `GITHUB_TOKEN`，需要启用 Actions 的 `packages: write` 权限。

```bash
docker run --rm --init -it ghcr.io/lyonbrown4d/terman-screen:latest
docker run --rm --init -it ghcr.io/lyonbrown4d/terman-tmux:latest
docker run --rm -it ghcr.io/lyonbrown4d/terman-htop:latest
```

直接以 detached 客户端作为容器主进程时，客户端退出会让容器停止。
需要保留会话时，启动持续运行的容器后用 `docker exec -it` 操作，并确保挂载目录允许 UID 10001 写入。
htop 默认观察容器中的进程；查看宿主机 PID 可显式加 `--pid=host`，操作权限仍受用户权限限制。
Snapshot 不上传镜像，buildx 将每个架构的镜像加载到本地，标签为 `v0.0.0-dev-COMMIT-amd64` / `-arm64`。
CI 对两个架构运行版本查询、Screen/tmux 会话创建与 IPC 查询、htop 单次采样。

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

## Native supervisor services

Screen and tmux can each run a native per-user supervisor. Sessions remain separate
processes; neither tool calls the other. Htop has no background service.

```text
terman-screen service install
terman-screen service start
terman-screen service status --json
terman-screen service stop
terman-screen service uninstall
```

The same commands are available with `terman-tmux`. `service run` runs the supervisor
in the foreground. Session creation uses a reachable supervisor automatically and
falls back to direct spawning only when no supervisor is available. Supervisor
errors are reported rather than silently bypassed.

Linux uses systemd **user** units, never a root terminal service. DEB/RPM packages
install units under `/usr/lib/systemd/user`; they do not enable services for every
account automatically. Activate the packaged unit as the intended session owner:

```sh
systemctl --user enable --now terman-screen.service
systemctl --user enable --now terman-tmux.service
```

On upgrade, package hooks reload running user managers and restart only services
that were already active. Removal stops and disables the packaged services while
retaining user configuration. Offline installations without a user manager are
supported; activation happens later in the user's login session. To keep a user
manager running after logout, an administrator may explicitly enable systemd
lingering for that account.

Windows uses SCM under an explicitly selected regular user account. Built-in
system/service accounts are not supported. Installation requires administrative
rights; the account must have the Windows log-on-as-a-service right. CLI account
installation reads the password from an environment variable, not an argument:

```powershell
# Set TERMAN_SERVICE_PASSWORD securely in this process before invoking install.
terman-screen service install --username 'COMPUTER\user' --password-env TERMAN_SERVICE_PASSWORD
terman-screen service start
```

Windows MSI packages are available for amd64 and arm64. Tools are independent
features; ScreenService and TmuxService are optional features. The installer grants
log-on-as-a-service and limited service-control permissions to the selected owner,
registers/starts selected services, and stops/removes them on uninstall. Password
properties are hidden in MSI logs. Configuration and user files are not deleted.

Clients must run as the same user as their supervisor. Named Pipes/Unix sockets
are scoped by SID/UID and protected by owner permissions. A separate instance can
use `service ... --name terman-screen-example` and matching client environment
`TERMAN_SERVICE_INSTANCE=example` (similarly for tmux).

Stopping a supervisor stops its managed session processes. Upgrades that restart a
service also terminate its live sessions; old PTYs cannot be restored by restarting
it. User configuration retention does not imply live-session persistence.

### Installer builds and lifecycle tests

GoReleaser builds compressed binaries, archives, DEB/RPM packages and Docker images.
Windows MSI construction is a separate WiX 7 stage using those compressed ZIP
payloads; it does not require GoReleaser Pro. Build outputs stay outside this repo.
MSIs include separate SHA-256 checksum files.

```powershell
./packaging/windows/Build-Msi.ps1 -ReleaseDirectory ../terman-release
```

WiX 7.0.0 and its matching Util extension must be installed outside the repository.
Release CI validates MSI native tables and UPX payloads on both Windows architectures.
Actual installer lifecycle tests are opt-in via `workflow_dispatch` with
`installer_lifecycle=true`, using disposable runners, test accounts and increasing
package versions. They cover feature selection, native service identity, real
session IPC, upgrade, removal and configuration retention. Do not run these
host-changing tests on a workstation containing an existing Terman installation.

WiX 7 requires explicit EULA acceptance and has OSMF maintenance-fee conditions
for certain revenue-generating uses. Read the [official WiX terms guidance](https://docs.firegiant.com/wix/osmf/)
before enabling MSI builds. `Build-Msi.ps1 -AcceptWixEula` is an explicit acceptance
option; CI uses the repository variable `TERMAN_ACCEPT_WIX_EULA=true` only after
the responsible person has accepted the terms. The default must not accept them
on a user's behalf.
