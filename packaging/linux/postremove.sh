#!/bin/sh
set -eu
command -v systemctl >/dev/null 2>&1 || exit 0
command -v runuser >/dev/null 2>&1 || exit 0
[ -d /run/systemd/system ] || exit 0
for runtime in /run/user/[0-9]*; do
    [ -S "$runtime/bus" ] || continue
    uid=${runtime##*/}
    case "$uid" in ''|*[!0-9]*|0) continue ;; esac
    account=$(getent passwd "$uid" | cut -d: -f1)
    [ -n "$account" ] || continue
    runuser -u "$account" -- env XDG_RUNTIME_DIR="$runtime" \
        DBUS_SESSION_BUS_ADDRESS="unix:path=$runtime/bus" \
        systemctl --user daemon-reload >/dev/null 2>&1 || :
done
