native-tool-not-found = 当前平台未检测到 {$tool}。请安装本机 {$tool} 可执行文件，或在可用时使用内置实现。
builtin-htop-cli-about = 跨平台 htop 风格终端监控工具。
builtin-htop-tab-processes = 进程
builtin-htop-tab-io = I/O
builtin-htop-tab-network = 网络
builtin-htop-help = F1 帮助，Tab/左/右 切换标签，方向键选择，PgUp/PgDn 滚动，1-4 跳转标签，F3 查找，F4 或 / 过滤，F5/t 树视图，F6/s 排序，F9 结束进程，F10/q/Esc 退出，+/- 调整刷新间隔。
builtin-screen-no-sessions = 未发现内置 screen 会话。使用 `terman-screen -S <name>` 创建命名会话。
builtin-screen-cli-about = 跨平台 screen 终端会话工具（自实现内置后端）。
builtin-screen-cli-examples =
    常见用法示例：
      - terman-screen
      - terman-screen -S dev
      - terman-screen --list
      - terman-screen -ls
      - terman-screen -d -S dev
      - terman-screen -d -m -S dev
      - terman-screen -dmS dev
      - terman-screen -D -r dev
      - terman-screen -d -r dev
      - terman-screen -R dev
      - terman-screen -wipe
      - terman-screen -S dev -X quit
      - terman-screen -S dev -Q info
      - terman-screen -S dev -X dinfo
      - terman-screen -S dev -X dumptermcap
      - terman-screen -S dev -X lastmsg
      - terman-screen -S dev -X redisplay
      - terman-screen -S dev -X displays
      - terman-screen -S dev -X windows
      - terman-screen -S dev -X windowlist
      - terman-screen -S dev -X hardcopy
      - terman-screen -S dev -X hardcopy -h
      - terman-screen -S dev -X hardcopydir .
      - terman-screen -S dev -X hardcopy_append on
      - terman-screen -S dev -X screen
      - terman-screen -S dev -X chdir .
      - terman-screen -S dev -X setenv EDITOR vim
      - terman-screen -S dev -X unsetenv EDITOR
      - terman-screen -S dev -X shelltitle shell
      - terman-screen -S dev -X term xterm-256color
      - terman-screen -S dev -X title editor
      - terman-screen -S dev -X select 0
      - terman-screen -S dev -X select -
      - terman-screen -S dev -X number
      - terman-screen -S dev -X number +1
      - terman-screen -S dev -X fit
      - terman-screen -S dev -X width 132
      - terman-screen -S dev -X height 42
      - terman-screen -S dev -X defscrollback 2000
      - terman-screen -S dev -X logfile screen.log
      - terman-screen -S dev -X logfile flush 10
      - terman-screen -S dev -X log on
      - terman-screen -S dev -X deflog on
      - terman-screen -S dev -X logtstamp after 120
      - terman-screen -S dev -X next
      - terman-screen -S dev -X prev
      - terman-screen -S dev -X previous
      - terman-screen -S dev -X other
      - terman-screen -X version
      - terman-screen -X license
      - terman-screen -X help
      - terman-screen -X commands
      - terman-screen -X time
      - terman-screen -S dev -X sleep 1
      - terman-screen -S dev -X at 0 info
      - terman-screen -S dev -X colon "info"
      - terman-screen -S dev -X source .screenrc
      - terman-screen -S dev -X readbuf input.txt
      - terman-screen -S dev -X readbuf -e utf-8 input.txt
      - terman-screen -S dev -X writebuf -e utf-8 output.txt
      - terman-screen -S dev -X readreg . input.txt
      - terman-screen -S dev -X readreg -e utf-8 . input.txt
      - terman-screen -S dev -X removebuf
      - terman-screen -S dev -X paste
      - terman-screen -S dev -X paste .
      - terman-screen -S dev -X process .
      - terman-screen -S dev -X register . "hello\015"
      - terman-screen -S dev -X register -e utf-8 . "hello\015"
      - terman-screen -S dev -X stuff "echo hi\n"
      - terman-screen -S dev -X meta
      - terman-screen -S dev -X xon
      - terman-screen -S dev -X xoff
      - terman-screen -S dev -p 0 -X stuff "echo hi\n"
      - terman-screen -r dev
      - terman-screen -x dev
builtin-screen-session-list-header = 内置 screen 会话:
builtin-screen-session-list-entry =   {$name} pid={$pid} attached_clients={$attach_clients} replay_bytes={$replay_bytes} size={$cols}x{$rows} cwd={$cwd} command={$command}
builtin-screen-session-exists = 内置 screen 会话 `{$name}` 已存在。请先使用 `terman-screen --list` 查看现有会话，或换一个会话名。
builtin-screen-session-name-empty = screen 会话名不能为空。
builtin-screen-session-record-invalid = 内置 screen 会话记录无效。
builtin-screen-unexpected-response = 非预期的 screen 响应：{$response}。
builtin-screen-attach-unsupported = 内置 screen 暂未开放 attach。跨平台 attach 将由内置会话服务处理。
builtin-screen-attach-help = screen 快捷键：Ctrl-A c/Ctrl-C 新建窗口 | Ctrl-A d/Ctrl-D 断开连接 | Ctrl-A D 断开全部连接 | Ctrl-A k/Ctrl-K 结束当前窗口 | Ctrl-A C 清屏 | Ctrl-A Z 重置终端 | Ctrl-A r 同步尺寸 | Ctrl-A l 重绘 | Ctrl-A m/Ctrl-M 显示最后消息 | Ctrl-A h 生成 hardcopy | Ctrl-A H 切换日志 | Ctrl-A . 生成 termcap | Ctrl-A ]/Ctrl-] 粘贴 paste buffer | Ctrl-A </>/= 交换 buffer | Ctrl-A q 发送 xon | Ctrl-A s 发送 xoff | Ctrl-A i/Ctrl-I 显示信息 | Ctrl-A n/Ctrl-N/Space 下一个窗口 | Ctrl-A N 显示窗口编号 | Ctrl-A p/Ctrl-P/Backspace 上一个窗口 | Ctrl-A 0..9 选择窗口 | Ctrl-A * 显示 displays | Ctrl-A t/Ctrl-T 显示时间 | Ctrl-A v 显示版本 | Ctrl-A , 显示 license | Ctrl-A F 适配尺寸 | Ctrl-A W 切换宽度 | Ctrl-A w/Ctrl-W 显示窗口 | Ctrl-A " 显示 windowlist | Ctrl-A \\ 退出 screen | Ctrl-A ? 显示帮助 | Ctrl-A A 设置标题 | Ctrl-A Ctrl-A 上一个活动窗口 | Ctrl-A a 发送字面 Ctrl-A
builtin-screen-attach-hardcopy-path-unavailable = 没有可用的 screen attach hardcopy 路径。
builtin-screen-attach-title-prompt = 窗口标题：
builtin-screen-attach-target-required = 请指定 screen 会话名；当前可能存在多个内置 screen 会话。
builtin-screen-attach-output-thread-panicked = screen attach 输出线程发生 panic。
builtin-screen-session-not-found = 未找到内置 screen 会话 `{$name}`。
builtin-screen-named-session-required = 启动命名 screen 会话需要指定会话名。
builtin-screen-server-timeout = 等待内置 screen 会话服务启动超时。
builtin-screen-service-timeout = 内置 screen 会话服务未响应。
builtin-screen-internal-server-session-required = 内置 screen server 需要会话名。
builtin-screen-internal-server-exited = 内置 screen server 已退出，退出码 {$code}。
builtin-screen-failure = 内置 screen 执行失败，退出码 {$code}。
builtin-screen-control-command-required = 请指定 screen 控制命令。
builtin-screen-control-command-unsupported = 暂不支持 screen 控制命令 `{$command}`。目前支持：quit、kill、bell、help、commands、echo、wall、lastmsg、stuff、meta、xon、xoff、screen、shell、defshell、shelltitle、term、chdir、setenv、unsetenv、paste、pastefile、bufferfile、process、register、readreg、readbuf、removebuf、writebuf、source、detach、pow_detach、resize、fit、width、height、select、number、next、prev、previous、other、scrollback、defscrollback、logfile、log、deflog、logtstamp、title、aka、sleep、time、version、license、info、dinfo、dumptermcap、displays、windows、windowlist、hardcopy、hardcopydir、hardcopy_append、clear、reset、redisplay、eval、at、colon、sessionname。
builtin-screen-control-chdir-directory-required = 请按 screen -X chdir <路径> 指定一个已存在的目录。
builtin-screen-control-chdir-home-required = screen -X chdir 无法找到 HOME 或 USERPROFILE。
builtin-screen-control-echo-required = 请为 screen -X echo 或 screen -X wall 指定要广播的文本。
builtin-screen-control-lastmsg-empty = 暂无上一条 screen 消息。
builtin-screen-control-setenv-required = 请按 screen -X setenv <名称> <值> 指定环境变量和值。
builtin-screen-control-unsetenv-required = 请按 screen -X unsetenv <名称> 指定环境变量名。
builtin-screen-control-env-name-invalid = 环境变量名不能为空，也不能包含 =。
builtin-screen-control-shell-required = 请按 screen -X shell <命令> 指定默认 shell 命令。
builtin-screen-control-shelltitle-required = 请按 screen -X shelltitle <标题> 指定默认 shell 窗口标题。
builtin-screen-control-term-required = 请按 screen -X term <名称> 指定默认终端类型。
builtin-screen-control-log-required = 请按 screen -X log [on|off] 指定日志状态；省略状态时切换日志开关。
builtin-screen-control-logfile-required = 请按 screen -X logfile <路径> 指定日志文件路径，或按 screen -X logfile flush <秒数> 指定刷新间隔。
builtin-screen-control-logtstamp-required = 请按 logtstamp [on|off]、logtstamp after <秒数> 或 logtstamp string <文本> 指定日志时间戳。
builtin-screen-control-help = 支持的 screen -X 命令：quit、kill、bell、help、commands、echo、wall、lastmsg、stuff、meta、xon、xoff、screen、shell、defshell、shelltitle、term、chdir、setenv、unsetenv、paste、pastefile、bufferfile、process、register、readreg、readbuf、removebuf、writebuf、source、detach、pow_detach、resize、fit、width、height、select、number、next、prev、previous、other、scrollback、defscrollback、logfile、log、deflog、logtstamp、title、aka、sleep、time、version、license、info、dinfo、dumptermcap、displays、windows、windowlist、hardcopy、hardcopydir、hardcopy_append、clear、reset、redisplay、eval、at、colon、sessionname。
builtin-screen-control-stuff-required = 请为 screen -X stuff 指定要输入的文本。
builtin-screen-control-register-required = 请按 screen -X register [-e encoding] <寄存器> <文本> 指定寄存器文本。
builtin-screen-control-resize-required = 请按 `screen -X resize <列数> <行数>` 指定 resize 尺寸。
builtin-screen-control-select-unsupported = 不支持的 screen 窗口 selector `{$selector}`。请使用可见的数字窗口索引、标题、-、.、# 或空 selector。
builtin-screen-control-number = screen 窗口编号：{$index} {$title}
builtin-screen-control-number-invalid = 请按 screen -X number [index|+delta|-delta] 指定 screen 窗口编号。
builtin-screen-control-scrollback-required = 请按整数行数指定 scrollback：screen -X defscrollback <行数>。
builtin-screen-control-sleep-required = 请按整数秒数指定 sleep 时长：screen -X sleep <秒数>。
builtin-screen-control-time = screen 时间：unix_seconds={$unix_seconds}
builtin-screen-control-title-required = 请按 screen -X title <标题> 指定当前窗口标题。
builtin-screen-control-version = terman-screen {$version} 内置跨平台后端
builtin-screen-control-license = terman-screen {$version} 内置后端。本项目独立实现 GNU Screen 兼容命令；再分发条款和免责声明请以仓库 license 为准。
builtin-screen-control-info = screen 信息：session={$session_name} replay_bytes={$replay_bytes} attach_clients={$attach_clients} size={$cols}x{$rows} scrollback_lines={$scrollback_lines}
builtin-screen-control-dinfo = screen 显示信息：session={$session_name} size={$cols}x{$rows} active_window={$active_window} attached_clients={$attach_clients} term={$term}
builtin-screen-control-displays-entry = displays：session={$session_name} attached_clients={$attach_clients} size={$cols}x{$rows}
builtin-screen-control-windows-entry = {$index}{$active_marker} {$title} size={$cols}x{$rows} attach_clients={$attach_clients} replay_bytes={$replay_bytes}
builtin-screen-control-unexpected-response = 非预期的 screen 控制响应：{$response}。
builtin-screen-control-hardcopy-path-required = 可按 screen -X hardcopy [-h] [路径] 指定输出路径；省略时写入 hardcopy.<窗口编号>。
builtin-screen-control-hardcopydir-required = 请按 screen -X hardcopydir <路径> 指定一个已存在的目录。
builtin-screen-control-hardcopy-append-required = 请按 screen -X hardcopy_append <on|off> 指定 hardcopy 追加写入状态。
builtin-screen-control-pastefile-path-required = 请按 screen -X pastefile <路径> 指定输入文件路径。
builtin-screen-control-readbuf-path-required = 可以用 screen -X readbuf [-e encoding] [path] 指定可选输入路径；省略路径时使用 screen 交换文件。
builtin-screen-control-readreg-required = 请按 screen -X readreg [-e encoding] <寄存器> <路径> 指定寄存器和输入文件路径。
builtin-screen-control-writebuf-path-required = 可以用 screen -X writebuf [-e encoding] [path] 指定可选输出路径；省略路径时使用 screen 交换文件。
builtin-screen-control-buffer-encoding-required = 请按 screen -X readbuf -e <encoding> [path]、screen -X writebuf -e <encoding> [path]、screen -X readreg -e <encoding> <寄存器> <路径> 或 screen -X register -e <encoding> <寄存器> <文本> 指定受支持的编码。
builtin-screen-control-source-path-required = 请按 screen -X source <路径> 指定命令文件路径。
builtin-screen-control-hardcopy-complete = 已将 {$bytes} 字节 screen hardcopy 写入 {$path}。
builtin-screen-control-dumptermcap-complete = 已将 screen termcap 条目写入 {$path}。
builtin-screen-control-writebuf-complete = 已将 {$bytes} 字节 screen paste buffer 写入 {$path}。
builtin-screen-wipe-complete = 已清理 {$count} 个失效的内置 screen 会话记录。
builtin-tmux-no-sessions = 当前没有 tmux 会话
builtin-tmux-cli-about = 跨平台 tmux 终端会话工具（自实现内置后端）。
builtin-tmux-cli-examples =
    常见用法示例：
      - terman-tmux new -s dev
      - terman-tmux new-session -s dev
      - terman-tmux attach -t <session>
      - terman-tmux attach-session -t <session>
      - terman-tmux list-sessions
      - terman-tmux list-clients
      - terman-tmux list-windows -t dev
      - terman-tmux --detached new -s dev

    排查示例：
      - 会话不存在：terman-tmux attach -t missing-session
      - 先查看会话：terman-tmux list-sessions
      - 名称冲突：terman-tmux new -s demo
      - 再复现冲突：terman-tmux new -s demo
builtin-tmux-attach-help = tmux 快捷键：Ctrl-B c 新建窗口 | Ctrl-B d 断开连接 | Ctrl-B x/& 结束窗口 | Ctrl-B , 重命名窗口 | Ctrl-B n/p 前后切换 | Ctrl-B l 上一个窗口 | Ctrl-B 0..9 选择窗口 | Ctrl-B ? 帮助
builtin-tmux-attach-window-list = 窗口：{$windows}
builtin-tmux-session-list-entry = {$name}：{$windows} 个窗口（已连接 {$attached_clients} 个客户端）
builtin-tmux-client-list-entry = {$session}：已连接 {$attached_clients} 个客户端
builtin-tmux-window-list-entry = {$session}:{$index}: {$name}
builtin-tmux-pane-list-entry = {$session}:{$window}.{$pane}: {$name} active={$active}
builtin-tmux-session-killed = 已结束 tmux 会话 {$name}
builtin-tmux-session-not-found = 未找到 tmux 会话 {$name}
builtin-tmux-target-required = 请使用 -t <名称> 指定目标会话
builtin-tmux-session-created = 已创建 tmux 会话 {$name}
builtin-tmux-session-exists = tmux 会话 {$name} 已存在
builtin-tmux-session-name-required = 请使用 -s <名称> 指定会话名称
builtin-tmux-command-unsupported = 内置 tmux 暂不支持命令 {$command}。当前不会调用系统 tmux；请等待后续跨平台实现。
builtin-tmux-internal-server-session-required = 内置 tmux server 需要会话名。
builtin-tmux-internal-server-exited = 内置 tmux server 已退出，退出码 {$code}。
builtin-tmux-server-not-responding = 内置 tmux server 未响应。
builtin-tmux-server-not-ready = 内置 tmux server 未就绪。
builtin-tmux-unexpected-info-response = 非预期的 tmux info 响应：{$response}。
builtin-tmux-unexpected-response = 非预期的 tmux 响应：{$response}。
builtin-tmux-message-required = 请为 tmux display-message 指定消息文本。
builtin-tmux-keys-required = 请为 tmux send-keys 指定按键或文本。
builtin-tmux-window-created = 已在 tmux 会话 {$session} 中创建窗口，当前共 {$windows} 个窗口
builtin-tmux-window-killed = 已结束 tmux 会话 {$session} 中的一个窗口，当前剩余 {$windows} 个窗口
builtin-tmux-window-name-required = 请指定新的 tmux 窗口名称
builtin-tmux-window-not-found = 未找到 tmux 会话 {$session} 中的窗口 {$index}
