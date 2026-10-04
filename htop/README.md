# terman-htop

Native Windows/Linux process monitor using tcell and gopsutil.

Build: `go build ./cmd/terman-htop`

Usage: `terman-htop [--refresh-ms 1000] [--once] [--sort cpu|memory|io|pid|name] [--reverse] [--filter text]`

The full-screen UI keeps Overview, Processes, IO, and Network tabs. `Tab`/`Shift-Tab`, arrows, number keys, and mouse clicks navigate.

htop-style controls:

- `F2` setup, `F3` or `/` search, `F4` or `\` live filter, `F5` tree, `F6` sort.
- `F7`/`F8` adjust priority; `F9` opens the signal menu and requires confirmation. `k`/`K` confirm TERM/KILL directly.
- `Space` tags a process, `U` clears tags, `u` selects an exact user, and `F` follows the selected PID.
- `-`/`+` collapse or expand a tree node; `*` collapses or expands all nodes.
- `d` opens scrolling process details; `e` opens the full-screen scrolling environment view.
- Mouse wheel moves selection or scrolls overlays/views. Left click selects controls/rows, middle click tags, and right click opens details.
- `F10`, `q`, or `Ctrl-C` exits and restores the terminal.
