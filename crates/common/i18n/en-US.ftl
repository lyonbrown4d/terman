native-tool-not-found = {$tool} was not found on this platform. Install a native {$tool} executable or use the built-in implementation when available.
builtin-htop-cli-about = Cross-platform htop-style terminal monitor.
builtin-htop-tab-overview = Overview
builtin-htop-tab-processes = Processes
builtin-htop-tab-io = I/O
builtin-htop-tab-network = Network
builtin-htop-help = F1 help, Tab/Left/Right switch tabs, arrows select, PgUp/PgDn scroll, 1-4 jump tabs, F3 search, F4 or / filter, F5/t tree, F6/s sort menu, I invert sort order, F9 choose signal, F10/q/Esc quit, +/- adjust refresh. Mouse: click tabs/footer/Overview or Processes rows/table headers, wheel lists/details/I/O/Network, right-click process signal menu., F7/F8 adjust priority
builtin-htop-help-panel =
    terman htop

    F1 Help: toggle this panel.
    Tab or Left/Right: switch Overview, Processes, I/O, Network tabs.
    1-4: jump to a tab directly.
    Arrows, Home, End, PgUp, PgDn: move process selection.
    F3: search visible processes.
    F4 or /: filter processes by PID, name, or command line.
    F5 or t: toggle flat/tree process view.
    F6 or s: open the sort menu.
    F7/F8: decrease/increase nice for the selected process.
    I: invert the current sort order.
    F9: choose and send a supported signal to the selected process.
    +/-: adjust refresh delay.
    F10, q, or Esc: quit.
    Mouse: click tabs, footer actions, Overview TOP PROCESSES rows, Processes rows, and Processes table headers.
    Mouse wheel: move process selection; over details, I/O, or Network, scroll that view.
    Right-click process row: open the signal menu.
builtin-htop-sort-menu-title = Sort by
builtin-htop-sort-menu-help = Up/Down choose, Enter apply, Esc cancel.
builtin-htop-signal-menu-title = Send signal to PID {$pid}
builtin-htop-signal-menu-help = Up/Down or wheel choose, Enter or click send, Esc/F9 cancel.
builtin-htop-signal-unsupported = No process signals are supported on this platform.
builtin-htop-signal-footer =  signal PID {$pid}:
builtin-screen-no-sessions = No built-in screen sessions found. Use `terman-screen -S <name>` to create a named session.
builtin-screen-cli-about = Cross-platform screen terminal session tool with a built-in backend.
builtin-screen-cli-examples =
    Common examples:
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
builtin-screen-session-list-header = Built-in screen sessions:
builtin-screen-session-list-entry =   {$name} pid={$pid} attached_clients={$attach_clients} replay_bytes={$replay_bytes} size={$cols}x{$rows} cwd={$cwd} command={$command}
builtin-screen-session-exists = Built-in screen session `{$name}` already exists. Run `terman-screen --list` to inspect existing sessions, or choose another name.
builtin-screen-session-name-empty = Screen session name cannot be empty.
builtin-screen-session-record-invalid = Built-in screen session record is invalid.
builtin-screen-unexpected-response = Unexpected screen response: {$response}.
builtin-screen-attach-unsupported = Built-in screen attach is not available yet. Cross-platform attach will be handled by the built-in session service.
builtin-screen-attach-help = screen keys: Ctrl-A c/Ctrl-C new window | Ctrl-A d/Ctrl-D detach | Ctrl-A D detach all | Ctrl-A k/Ctrl-K kill current window | Ctrl-A C clear | Ctrl-A Z reset | Ctrl-A r sync size | Ctrl-A l redisplay | Ctrl-A m/Ctrl-M last message | Ctrl-A M monitor | Ctrl-A h hardcopy | Ctrl-A H log | Ctrl-A . dumptermcap | Ctrl-A ]/Ctrl-] paste buffer | Ctrl-A </>/= exchange buffer | Ctrl-A q xon | Ctrl-A s xoff | Ctrl-A i/Ctrl-I info | Ctrl-A n/Ctrl-N/Space next | Ctrl-A N number | Ctrl-A p/Ctrl-P/Backspace previous | Ctrl-A 0..9 select window | Ctrl-A * displays | Ctrl-A t/Ctrl-T time | Ctrl-A v version | Ctrl-A , license | Ctrl-A F fit | Ctrl-A W width | Ctrl-A w/Ctrl-W windows | Ctrl-A " windowlist | Ctrl-A \\ quit | Ctrl-A ? help | Ctrl-A A title | Ctrl-A Ctrl-A last window | Ctrl-A a send literal Ctrl-A | Mouse wheel next/previous | right-click windows | middle-click help | Ctrl-A [/Esc copy mode
builtin-screen-monitor-status =
    { $state ->
        [on] screen: monitoring window {$window}
       *[off] screen: no longer monitoring window {$window}
    }
builtin-screen-monitor-activity = screen: activity in window {$window} ({$title})builtin-screen-attach-hardcopy-path-unavailable = No available screen attach hardcopy path.
builtin-screen-attach-title-prompt = Window title:
builtin-screen-attach-target-required = Specify a screen session name when more than one built-in screen session may exist.
builtin-screen-attach-output-thread-panicked = Screen attach output thread panicked.
builtin-screen-session-not-found = Built-in screen session `{$name}` was not found.
builtin-screen-named-session-required = Named screen session launch requires a session name.
builtin-screen-server-timeout = Timed out waiting for the built-in screen session server.
builtin-screen-service-timeout = Built-in screen session service did not respond.
builtin-screen-internal-server-session-required = Internal screen server requires a session name.
builtin-screen-internal-server-exited = Internal screen server exited with code {$code}.
builtin-screen-failure = Built-in screen failed with exit code {$code}.
builtin-screen-control-command-required = Specify a screen control command.
builtin-screen-control-command-unsupported = Unsupported screen control command `{$command}`. Currently supported: quit, kill, bell, help, commands, echo, wall, lastmsg, monitor, stuff, meta, xon, xoff, screen, shell, defshell, shelltitle, term, chdir, setenv, unsetenv, paste, pastefile, bufferfile, process, register, readreg, readbuf, removebuf, writebuf, source, detach, pow_detach, resize, fit, width, height, select, number, next, prev, previous, other, scrollback, defscrollback, logfile, log, deflog, logtstamp, title, aka, sleep, time, version, license, info, dinfo, dumptermcap, displays, windows, windowlist, hardcopy, hardcopydir, hardcopy_append, clear, reset, redisplay, eval, at, colon, sessionname, split, focus, remove, only.
builtin-screen-control-chdir-directory-required = Specify an existing directory as screen -X chdir <path>.
builtin-screen-control-chdir-home-required = HOME or USERPROFILE is not available for screen -X chdir.
builtin-screen-control-echo-required = Specify text for screen -X echo or screen -X wall.
builtin-screen-control-lastmsg-empty = No previous screen message.
builtin-screen-control-setenv-required = Specify environment variable and value as screen -X setenv <name> <value>.
builtin-screen-control-unsetenv-required = Specify environment variable name as screen -X unsetenv <name>.
builtin-screen-control-env-name-invalid = Environment variable names cannot be empty or contain =.
builtin-screen-control-shell-required = Specify a shell command as screen -X shell <command>.
builtin-screen-control-shelltitle-required = Specify a default shell window title as screen -X shelltitle <title>.
builtin-screen-control-term-required = Specify a terminal type as screen -X term <name>.
builtin-screen-control-log-required = Specify screen log state as screen -X log [on|off], or omit the state to toggle logging.
builtin-screen-control-monitor-required = Use screen -X monitor [on|off|toggle].builtin-screen-control-logfile-required = Specify a log file path as screen -X logfile <path>, or a flush interval as screen -X logfile flush <seconds>.
builtin-screen-control-logtstamp-required = Specify screen log timestamp as logtstamp [on|off], logtstamp after <seconds>, or logtstamp string <text>.
builtin-screen-control-help = Supported screen -X commands: quit, kill, bell, help, commands, echo, wall, lastmsg, monitor, stuff, meta, xon, xoff, screen, shell, defshell, shelltitle, term, chdir, setenv, unsetenv, paste, pastefile, bufferfile, process, register, readreg, readbuf, removebuf, writebuf, source, detach, pow_detach, resize, fit, width, height, select, number, next, prev, previous, other, scrollback, defscrollback, logfile, log, deflog, logtstamp, title, aka, sleep, time, version, license, info, dinfo, dumptermcap, displays, windows, windowlist, hardcopy, hardcopydir, hardcopy_append, clear, reset, redisplay, eval, at, colon, sessionname, split, focus, remove, only.
builtin-screen-control-stuff-required = Specify text for screen -X stuff.
builtin-screen-control-register-required = Specify register text as screen -X register [-e encoding] <register> <text>.
builtin-screen-control-resize-required = Specify resize dimensions as `screen -X resize <cols> <rows>`.
builtin-screen-control-select-unsupported = Unsupported screen window selector `{$selector}`. Use a visible numeric window index, title, -, ., #, or an empty selector.
builtin-screen-control-number = screen window number: {$index} {$title}
builtin-screen-control-number-invalid = Specify a screen window number as screen -X number [index|+delta|-delta].
builtin-screen-control-scrollback-required = Specify scrollback as integer lines: screen -X defscrollback <lines>.
builtin-screen-control-sleep-required = Specify sleep duration as integer seconds: screen -X sleep <seconds>.
builtin-screen-control-time = screen time: unix_seconds={$unix_seconds}
builtin-screen-control-title-required = Specify a current window title as screen -X title <title>.
builtin-screen-control-version = terman-screen {$version} built-in cross-platform backend
builtin-screen-control-license = terman-screen {$version} built-in backend. This project implements GNU Screen-compatible commands independently; see the repository license for redistribution terms and warranty disclaimer.
builtin-screen-control-info = screen info: session={$session_name} replay_bytes={$replay_bytes} attach_clients={$attach_clients} size={$cols}x{$rows} scrollback_lines={$scrollback_lines}
builtin-screen-control-dinfo = screen display info: session={$session_name} size={$cols}x{$rows} active_window={$active_window} attached_clients={$attach_clients} term={$term}
builtin-screen-control-displays-entry = displays: session={$session_name} attached_clients={$attach_clients} size={$cols}x{$rows}
builtin-screen-control-windows-entry = {$index}{$active_marker} {$title} size={$cols}x{$rows} attach_clients={$attach_clients} replay_bytes={$replay_bytes}
builtin-screen-control-unexpected-response = Unexpected screen control response: {$response}.
builtin-screen-control-hardcopy-path-required = Specify an optional output path as screen -X hardcopy [-h] [path].
builtin-screen-control-hardcopydir-required = Specify an existing directory as screen -X hardcopydir <path>.
builtin-screen-control-hardcopy-append-required = Specify hardcopy append state as screen -X hardcopy_append <on|off>.
builtin-screen-control-pastefile-path-required = Specify an input path as screen -X pastefile <path>.
builtin-screen-control-readbuf-path-required = Specify an optional input path as screen -X readbuf [-e encoding] [path]; omit the path to use the screen exchange file.
builtin-screen-control-readreg-required = Specify register and input path as screen -X readreg [-e encoding] <register> <path>.
builtin-screen-control-writebuf-path-required = Specify an optional output path as screen -X writebuf [-e encoding] [path]; omit the path to use the screen exchange file.
builtin-screen-control-buffer-encoding-required = Specify a supported encoding as screen -X readbuf -e <encoding> [path], screen -X writebuf -e <encoding> [path], screen -X readreg -e <encoding> <register> <path>, or screen -X register -e <encoding> <register> <text>.
builtin-screen-control-source-path-required = Specify a command file path as screen -X source <path>.
builtin-screen-control-hardcopy-complete = Wrote {$bytes} byte(s) of screen hardcopy to {$path}.
builtin-screen-control-dumptermcap-complete = Wrote screen termcap entry to {$path}.
builtin-screen-control-writebuf-complete = Wrote {$bytes} byte(s) of screen paste buffer to {$path}.
builtin-screen-wipe-complete = Removed {$count} stale built-in screen session record(s).
builtin-tmux-no-sessions = no tmux sessions
builtin-tmux-cli-about = Cross-platform tmux terminal session tool with a built-in backend.
builtin-tmux-cli-examples =
    Common examples:
      - terman-tmux new -s dev
      - terman-tmux new-session -s dev
      - terman-tmux attach -t <session>
      - terman-tmux attach-session -t <session>
      - terman-tmux list-sessions
      - terman-tmux list-clients
      - terman-tmux list-windows -t dev
      - terman-tmux --detached new -s dev

    Troubleshooting examples:
      - Missing session: terman-tmux attach -t missing-session
      - List sessions first: terman-tmux list-sessions
      - Name conflict: terman-tmux new -s demo
      - Reproduce conflict: terman-tmux new -s demo
builtin-tmux-attach-help = tmux keys: Ctrl-B c new window | Ctrl-B d detach | Ctrl-B %/" split pane | Ctrl-B o next pane | Ctrl-B {/} swap pane up/down | Ctrl-B z zoom pane | Ctrl-B x kill pane | Ctrl-B & kill window | Ctrl-B , rename window | Ctrl-B n/p next/previous | Ctrl-B l last window | Ctrl-B 0..9 select window | Ctrl-B ? help | Mouse: status click/wheel switch, right-click windows, middle-click help | Ctrl-B [ copy mode | Ctrl-B ] paste buffer
builtin-tmux-prefix-status = tmux prefix | %/" split | o next pane | z zoom pane | x kill pane | & kill window | d detach
builtin-tmux-attach-window-list = windows: {$windows}
builtin-tmux-session-list-entry = {$name}: {$windows} windows (attached {$attached_clients})
builtin-tmux-client-list-entry = {$session}: {$attached_clients} attached client(s)
builtin-tmux-window-list-entry = {$session}:{$index}: {$name}
builtin-tmux-pane-list-entry = {$session}:{$window}.{$pane}: {$name} active={$active}
builtin-tmux-pane-not-found = tmux pane {$window}.{$pane} in session {$session} was not found
builtin-tmux-pane-size-required = specify pane size with resize-pane -x <cols> -y <rows>
builtin-tmux-session-killed = killed tmux session {$name}
builtin-tmux-session-not-found = tmux session {$name} not found
builtin-tmux-target-required = specify a target session with -t <name>
builtin-tmux-session-created = created tmux session {$name}
builtin-tmux-session-exists = tmux session {$name} already exists
builtin-tmux-session-name-required = specify a session name with -s <name>
builtin-tmux-command-unsupported = Built-in tmux command {$command} is not supported yet. This tool will not call the system tmux binary.
builtin-tmux-internal-server-session-required = Internal tmux server requires a session name.
builtin-tmux-internal-server-exited = Internal tmux server exited with code {$code}.
builtin-tmux-server-not-responding = Built-in tmux server did not respond.
builtin-tmux-server-not-ready = Built-in tmux server did not become ready.
builtin-tmux-unexpected-info-response = Unexpected tmux info response: {$response}.
builtin-tmux-unexpected-response = Unexpected tmux response: {$response}.
builtin-tmux-message-required = Specify a message for tmux display-message.
builtin-tmux-keys-required = Specify keys for tmux send-keys.
builtin-tmux-window-created = created a window in tmux session {$session}; {$windows} window(s) now exist
builtin-tmux-window-killed = killed one window in tmux session {$session}; {$windows} window(s) remain
builtin-tmux-window-name-required = specify a new tmux window name
builtin-tmux-window-not-found = tmux window {$index} in session {$session} was not found

builtin-screen-copy-status = copy mode | line {$line}/{$total} | arrows/PgUp/PgDn move | Space/Enter mark | Esc cancel
builtin-screen-copy-selection-status = copy mode | line {$line}/{$total} | arrows/PgUp/PgDn move | Space/Enter copy | Esc cancel
builtin-tmux-copy-status = tmux copy mode | line {$line}/{$total} | arrows/PgUp/PgDn move | Space/Enter mark | Esc cancel
builtin-tmux-copy-selection-status = tmux copy mode | line {$line}/{$total} | arrows/PgUp/PgDn move | Space/Enter copy | Esc cancel
builtin-tmux-buffer-data-required = set-buffer requires buffer data.
builtin-tmux-buffer-list-item = {$name}: {$bytes} bytes: "{$preview}"
builtin-tmux-buffer-not-found = tmux buffer "{$name}" was not found.
builtin-tmux-buffer-unavailable = tmux buffer state is unavailable.
