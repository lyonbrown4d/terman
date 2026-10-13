#!/bin/sh
set -eu
# Debian upgrade/deconfigure and RPM $1=1 retain the current service state.
case "${1-}" in remove|0) ;; *) exit 0 ;; esac
exec /usr/libexec/terman-screen/user-lifecycle.sh screen removing
