#!/bin/sh
# Package hooks only address already running non-root user managers.
# They neither enable new services nor touch user configuration/session data.
set -eu
tool=${1:?tool required}
operation=${2:?operation required}
case "$tool" in screen|tmux) ;; *) exit 1 ;; esac
unit="terman-$tool.service"
command -v systemctl >/dev/null 2>&1 || exit 0
command -v runuser >/dev/null 2>&1 || exit 0
[ -d /run/systemd/system ] || exit 0
for runtime in /run/user/[0-9]*; do
    [ -S "$runtime/bus" ] || continue
    uid=${runtime##*/}
    case "$uid" in ''|*[!0-9]*|0) continue ;; esac
    account=$(getent passwd "$uid" | cut -d: -f1)
    [ -n "$account" ] || continue
    userctl() {
        runuser -u "$account" -- env XDG_RUNTIME_DIR="$runtime" \
            DBUS_SESSION_BUS_ADDRESS="unix:path=$runtime/bus" systemctl --user "$@"
    }
    # A logged-out user or inaccessible bus is an offline installation case.
    userctl show-environment >/dev/null 2>&1 || continue
    case "$operation" in
        installed)
            userctl daemon-reload
            # try-restart preserves an inactive service's state.
            userctl try-restart "$unit"
            ;;
        removing)
            userctl stop "$unit"
            # Remove only the package unit's enablement, retaining drop-ins/data.
            userctl disable "$unit"
            ;;
        removed) userctl daemon-reload >/dev/null 2>&1 || : ;;
        *) exit 1 ;;
    esac
done
