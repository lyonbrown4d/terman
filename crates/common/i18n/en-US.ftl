native-tool-not-found = {$tool} was not found on this platform. Install a native {$tool} executable or use the built-in implementation when available.
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
      - terman-screen -S dev -X displays
      - terman-screen -S dev -X windows
      - terman-screen -S dev -X select 0
      - terman-screen -X version
      - terman-screen -X help
      - terman-screen -S dev -X sleep 1
      - terman-screen -S dev -X at 0 info
      - terman-screen -S dev -X colon "info"
      - terman-screen -S dev -X source .screenrc
      - terman-screen -S dev -X stuff "echo hi\n"
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
builtin-screen-attach-help = screen keys: Ctrl-A d detach | Ctrl-A D detach all | Ctrl-A k kill session | Ctrl-A C clear | Ctrl-A Z reset | Ctrl-A r sync size | Ctrl-A h hardcopy | Ctrl-A i info | Ctrl-A * displays | Ctrl-A v version | Ctrl-A w windows | Ctrl-A ? help | Ctrl-A Ctrl-A send literal Ctrl-A
builtin-screen-attach-hardcopy-path-unavailable = No available screen attach hardcopy path.
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
builtin-screen-control-command-unsupported = Unsupported screen control command `{$command}`. Currently supported: quit, kill, bell, help, echo, wall, stuff, pastefile, source, detach, pow_detach, resize, select, sleep, version, info, displays, windows, hardcopy, clear, reset, eval, at, colon, sessionname.
builtin-screen-control-echo-required = Specify text for screen -X echo or screen -X wall.
builtin-screen-control-help = Supported screen -X commands: quit, kill, bell, help, echo, wall, stuff, pastefile, source, detach, pow_detach, resize, select, sleep, version, info, windows, hardcopy, clear, reset, eval, at, colon, sessionname.
builtin-screen-control-stuff-required = Specify text for screen -X stuff.
builtin-screen-control-resize-required = Specify resize dimensions as `screen -X resize <cols> <rows>`.
builtin-screen-control-select-unsupported = Built-in screen has one window; select supports only 0, ., #, or an empty selector. Got `{$selector}`.
builtin-screen-control-sleep-required = Specify sleep duration as integer seconds: screen -X sleep <seconds>.
builtin-screen-control-version = terman-screen {$version} built-in cross-platform backend
builtin-screen-control-info = screen info: session={$session_name} replay_bytes={$replay_bytes} attach_clients={$attach_clients} size={$cols}x{$rows}
builtin-screen-control-displays-entry = displays: session={$session_name} attached_clients={$attach_clients} size={$cols}x{$rows}
builtin-screen-control-windows-entry = 0* {$session_name} size={$cols}x{$rows} attach_clients={$attach_clients} replay_bytes={$replay_bytes}
builtin-screen-control-unexpected-response = Unexpected screen control response: {$response}.
builtin-screen-control-hardcopy-path-required = Specify an output path as screen -X hardcopy <path>.
builtin-screen-control-pastefile-path-required = Specify an input path as screen -X pastefile <path>.
builtin-screen-control-source-path-required = Specify a command file path as screen -X source <path>.
builtin-screen-control-hardcopy-complete = Wrote {$bytes} byte(s) of screen hardcopy to {$path}.
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
builtin-tmux-session-list-entry = {$name}: {$windows} windows (attached {$attached_clients})
builtin-tmux-client-list-entry = {$session}: {$attached_clients} attached client(s)
builtin-tmux-window-list-entry = {$session}:{$index}: {$name}
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
