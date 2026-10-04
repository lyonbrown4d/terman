# terman-htop

Native Windows/Linux process monitor using tcell and gopsutil.

Build: go build ./cmd/terman-htop

Usage: terman-htop [--refresh-ms 1000] [--once] [--sort cpu|memory|io|pid|name] [--reverse] [--filter text]

Keys: 1-4/arrows select tabs, j/k/arrows select rows, mouse selects and scrolls,
/ filters, c/m/i/p/n sorts, r reverses, d details, e environment, [ and ] priority,
k terminate, K force-kill, and q/F10/Ctrl-C exits.
