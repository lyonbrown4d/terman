native-tool-not-found = {$tool} was not found on this platform. Install a native {$tool} executable or use the built-in implementation when available.
builtin-screen-no-sessions = No built-in screen sessions found. Use `terman-screen -S <name>` to create a named session.
builtin-screen-session-list-header = Built-in screen sessions:
builtin-screen-session-exists = Built-in screen session `{$name}` already exists. Run `terman-screen --list` to inspect existing sessions, or choose another name.
builtin-screen-session-name-empty = Screen session name cannot be empty.
builtin-screen-attach-unsupported = Built-in screen attach is not available yet. Cross-platform attach will be handled by the built-in session service.
builtin-screen-attach-help = screen keys: Ctrl-A d detach | Ctrl-A k kill session | Ctrl-A C clear | Ctrl-A h hardcopy | Ctrl-A i info | Ctrl-A ? help | Ctrl-A Ctrl-A send literal Ctrl-A
builtin-screen-attach-target-required = Specify a screen session name when more than one built-in screen session may exist.
builtin-screen-session-not-found = Built-in screen session `{$name}` was not found.
builtin-screen-named-session-required = Named screen session launch requires a session name.
builtin-screen-server-timeout = Timed out waiting for the built-in screen session server.
builtin-screen-control-command-required = Specify a screen control command.
builtin-screen-control-command-unsupported = Unsupported screen control command `{$command}`. Currently supported: quit, kill, stuff, pastefile, detach, resize, info, hardcopy, clear, reset.
builtin-screen-control-stuff-required = Specify text for `screen -X stuff`.
builtin-screen-control-resize-required = Specify resize dimensions as `screen -X resize <cols> <rows>`.
builtin-screen-control-info = screen info: replay_bytes={$replay_bytes} attach_clients={$attach_clients} size={$cols}x{$rows}
builtin-screen-control-hardcopy-path-required = Specify an output path as screen -X hardcopy <path>.
builtin-screen-control-pastefile-path-required = Specify an input path as screen -X pastefile <path>.
builtin-screen-control-hardcopy-complete = Wrote {$bytes} byte(s) of screen hardcopy to {$path}.
builtin-screen-wipe-complete = Removed {$count} stale built-in screen session record(s).






