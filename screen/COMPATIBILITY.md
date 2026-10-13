# GNU Screen compatibility

This matrix tracks observable compatibility with the
[GNU Screen User's Manual](https://www.gnu.org/software/screen/manual/screen.html).
`Supported` means the behavior is covered by automated tests. `Partial` means the
command exists but does not yet implement every GNU Screen option or mode.

## Command-line compatibility

| GNU Screen operation | Terman Screen form | Status | Notes |
| --- | --- | --- | --- |
| New detached session | `-d -S name --command command` | Supported | Starts a native per-session server and PTY process. |
| Attach | `-r name` | Supported | Uses Named Pipe on Windows and Unix socket on Linux. |
| Multiattach | `-x name` | Supported | Multiple interactive clients can share a session. |
| List sessions | `-ls`, `-list` | Supported | Human and JSON output are available. |
| Wipe dead sessions | `-wipe` | Supported | Removes records whose server cannot be reached. |
| Execute command | `-S name -X command` | Supported | Sends the command to the live server. |
| Detach elsewhere | `-d -r name` | Partial | Attach behavior is available; GNU multi-display selection is incomplete. |
| Resume or create | `-R`, `-RR`, `-D -R` | Missing | Explicit create and attach are required. |
| Configuration file | `-c file`, `.screenrc` | Missing | Configuration uses the Terman config model. |
| Multiuser ACL | `-multiuser`, `-acl*` | Missing | Authentication and ACLs are not implemented. |
| Login terminal | `-L`, `-T`, `-U` | Partial | UTF-8 terminals work; GNU login/termcap options are incomplete. |

## Runtime command compatibility

| Command | Status | Notes |
| --- | --- | --- |
| `windows`, `windowlist` | Partial | Text window list is supported; GNU interactive chooser modes are not. |
| `screen` | Partial | Creates a window; GNU option parsing is incomplete. |
| `select`, `next`, `prev`, `previous`, `last`, `other` | Supported | Window selection is server-owned. |
| `kill`, `quit` | Supported | Kills a window or the complete session. |
| `title`, `aka` | Supported | Renames the active window. |
| `sessionname` | Supported | Renames the session record. |
| `info`, `dinfo`, `version`, `number`, `lastmsg` | Supported | Query commands return server state. |
| `help`, `commands` | Partial | Reports implemented commands, not the full GNU help UI. |
| `stuff` | Supported | Writes bytes to the active PTY. |
| `width`, `height` | Supported | Resizes the active terminal dimensions. |
| `split`, `focus`, `remove`, `only`, `resize` | Supported | Region layout and focus are server-owned. |
| `fit`, `redisplay` | Supported | Reflows or redraws attached displays. |
| `scrollback`, `defscrollback` | Supported | Active and default history limits are configurable. |
| `copy` | Supported | Enters copy mode through the client shortcut. |
| `register`, `readreg` | Partial | Named registers work; GNU encoding/file options are incomplete. |
| `readbuf`, `writebuf`, `removebuf`, `paste`, `bufferfile` | Supported | Exchange-buffer operations are available. |
| `pastefile` | Partial | Buffer file selection is supported without every GNU modifier. |
| `hardcopy`, `hardcopydir`, `hardcopy_append` | Supported | Captures terminal content to files. |
| `log`, `logfile`, `logtstamp`, `deflog` | Supported | Per-window logging is available. |
| `at`, `eval`, `exec`, `source`, `colon` | Missing | Command composition and external execution are not implemented. |
| `bind`, `bindkey`, `command`, `escape`, `meta` | Missing | Runtime key-map mutation is not implemented. |
| `acladd`, `aclchg`, `acldel`, `aclgrp`, `aclmode`, `aclumask`, `multiuser` | Missing | Multiuser ACL support is not implemented. |
| `activity`, `monitor`, `silence`, `silencewait`, `bell_msg`, `activity_msg` | Missing | Activity and silence monitoring are not implemented. |
| `backtick`, `caption`, `hardstatus`, `status`, `termcap`, `terminfo` | Missing | GNU status and termcap customization are not implemented. |
| `chdir`, `defchdir`, `setenv`, `unsetenv`, `shell`, `shelltitle` | Missing | Per-window runtime environment mutation is not implemented. |
| `console`, `login`, `deflogin` | Missing | Console grabbing and login accounting are not implemented. |
| `digraph`, `encoding`, `utf8`, `charset`, `cjkwidth` | Missing | GNU character-set conversion commands are not implemented. |
| `dumptermcap`, `term`, `termcapinfo`, `mapdefault`, `mapnotnext`, `maptimeout` | Missing | Terminal capability editing is not implemented. |
| `flow`, `defflow`, `interrupt`, `pow_break`, `break`, `breaktype` | Missing | Serial-line controls are not implemented. |
| `layout`, `layout new`, `layout save`, `layout select` | Missing | Regions exist, but named persistent layouts do not. |
| `lockscreen`, `password`, `blanker`, `blankerprg`, `idle` | Missing | Session locking and blanking are not implemented. |
| `markkeys`, `mousetrack`, `defmousetrack` | Missing | GNU copy-mode keymaps and mouse tracking options are not implemented. |
| `msgminwait`, `msgwait`, `msgwait`, `vbell`, `vbell_msg`, `vbellwait` | Missing | GNU message timing and visual bell controls are not implemented. |
| `nonblock`, `defnonblock`, `obuflimit`, `autodetach`, `startup_message` | Missing | GNU display buffering policy controls are not implemented. |
| `printcmd`, `screen`, `setenv`, `slowpaste`, `zombie`, `defzombie` | Partial | Core window creation exists; auxiliary modes are missing. |

## Default shortcut compatibility

The command prefix is `Ctrl-A`.

| Shortcut | Operation | Status |
| --- | --- | --- |
| `Ctrl-A d` | `detach` | Supported |
| `Ctrl-A c` | `screen` | Supported |
| `Ctrl-A Space`, `Ctrl-A n` | `next` | Supported |
| `Ctrl-A p` | `prev` | Supported |
| `Ctrl-A Ctrl-A` | `last` | Missing |
| `Ctrl-A w` | `windows` | Missing |
| `Ctrl-A A` | `title` | Supported |
| `Ctrl-A [` | `copy` | Supported |
| `Ctrl-A ]` | `paste` | Supported |
| `Ctrl-A S` | `split` | Supported |
| `Ctrl-A Tab` | `focus` | Supported |
| `Ctrl-A Q` | `only` | Supported |
| `Ctrl-A X` | `remove` | Supported |
| `Ctrl-A ?` | `help` | Supported |
| `Ctrl-A :` | command prompt | Supported |
| `Ctrl-A k` | `kill` | Supported |
| `Ctrl-A H` | `log` | Supported |
| `Ctrl-A M` | `monitor` | Missing |
| `Ctrl-A _` | `silence` | Missing |
| `Ctrl-A x` | `lockscreen` | Missing |

## Full GNU 5.0 command inventory

Source: [GNU Screen Command Index](https://www.gnu.org/software/screen/manual/html_node/Command-Index.html).
Each upstream command is classified as supported (`S`), partial (`P`), or missing (`M`).

- A: `acladd` M, `aclchg` M, `acldel` M, `aclgrp` M, `aclumask` M, `activity` M, `addacl` M, `allpartial` M, `altscreen` M, `at` M, `attrcolor` M, `autodetach` M, `autonuke` M.
- B: `backtick` M, `bce` M, `bell_msg` M, `bind` M, `bindkey` M, `blanker` M, `blankerprg` M, `break` M, `breaktype` M, `bufferfile` S, `bumpleft` M, `bumpright` M.
- C: `c1` M, `caption` M, `chacl` M, `charset` M, `chdir` M, `cjkwidth` M, `clear` M, `collapse` M, `colon` P, `command` P, `compacthist` M, `console` M, `copy` S, `copy_reg` M, `crlf` M.
- D: `defautonuke` M, `defbce` M, `defbreaktype` M, `defc1` M, `defcharset` M, `defencoding` M, `defescape` M, `defflow` M, `defgr` M, `defhstatus` M, `deflog` S, `deflogin` M, `defmode` M, `defmonitor` M, `defmousetrack` M, `defnonblock` M, `defobuflimit` M, `defscrollback` S, `defshell` M, `defsilence` M, `defslowpaste` M, `defutf8` M, `defwrap` M, `defwritelock` M, `detach` P, `digraph` M, `dinfo` S, `displays` M, `dumptermcap` M.
- E: `echo` M, `encoding` M, `escape` M, `eval` M, `exec` M.
- F: `fit` S, `flow` M, `focus` S, `focusminsize` M.
- G: `gr` M, `group` M.
- H: `hardcopy` S, `hardcopy_append` S, `hardcopydir` S, `hardstatus` M, `height` S, `help` P, `history` M, `hstatus` M.
- I: `idle` M, `ignorecase` M, `info` S, `ins_reg` M.
- K: `kill` S.
- L: `lastmsg` S, `layout` M, `license` M, `lockscreen` M, `log` S, `logfile` S, `login` M, `logtstamp` S.
- M: `mapdefault` M, `mapnotnext` M, `maptimeout` M, `markkeys` M, `meta` M, `monitor` M, `mousetrack` M, `msgminwait` M, `msgwait` M, `multiuser` M.
- N: `next` S, `nonblock` M, `number` S.
- O: `obuflimit` M, `only` S, `other` S.
- P: `parent` M, `partial` M, `password` M, `paste` S, `pastefont` M, `pow_break` M, `pow_detach` M, `pow_detach_msg` M, `prev` S, `printcmd` M, `process` M.
- Q: `quit` S.
- R: `readbuf` S, `readreg` P, `redisplay` S, `register` P, `remove` S, `removebuf` S, `rendition` M, `reset` M, `resize` S.
- S: `screen` P, `scrollback` S, `select` S, `sessionname` S, `setenv` M, `setsid` M, `shell` M, `shelltitle` M, `silence` M, `silencewait` M, `sleep` M, `slowpaste` M, `sorendition` M, `source` M, `split` S, `startup_message` M, `stuff` S, `su` M, `suspend` M.
- T: `term` M, `termcap` M, `termcapinfo` M, `terminfo` M, `title` S.
- U: `umask` M, `unbindall` M, `unsetenv` M, `utf8` M.
- V: `vbell` M, `vbell_msg` M, `vbellwait` M, `verbose` M, `version` S.
- W: `wall` M, `width` S, `windowlist` P, `windows` S, `wrap` M, `writebuf` S, `writelock` M.
- X: `xoff` M, `xon` M.
- Z: `zmodem` M, `zombie` M, `zombie_timeout` M.

## Validation

Run the process-level suite from the repository root:

```text
go test -tags=integration ./screen/e2e
```

The suite builds the real binary, starts native session servers, allocates real
PTYs, uses the platform IPC transport, exercises client crashes and stale record
cleanup, and runs attach/detach plus multi-client stress scenarios. Increase the
load with `TERMAN_SCREEN_E2E_SOAK`, `TERMAN_SCREEN_E2E_ATTACH_CYCLES`, and
`TERMAN_SCREEN_E2E_CLIENTS`.
