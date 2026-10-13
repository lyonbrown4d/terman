#!/bin/sh
set -eu
case "${1-}" in remove|0) ;; *) exit 0 ;; esac
exec /usr/libexec/terman-tmux/user-lifecycle.sh tmux removing
