# syntax=docker/dockerfile:1
FROM alpine:3.24.2@sha256:294b683cb724975bec92580e1e685676bd4b50bda910ddb8c51d4cabeaec77e6

# GoReleaser supplies the already-built, UPX-compressed binary for each platform.
ARG TARGETPLATFORM

# Alpine includes BusyBox /bin/sh; full terminfo supports interactive terminals.
RUN apk add --no-cache ncurses-terminfo \
    && addgroup -g 10001 terman \
    && adduser -D -u 10001 -G terman -h /home/terman -s /bin/sh terman \
    && mkdir -p /home/terman/.cache /home/terman/.run \
    && chown -R terman:terman /home/terman \
    && chmod 0700 /home/terman/.cache /home/terman/.run

ENV HOME=/home/terman \
    SHELL=/bin/sh \
    TERM=xterm-256color \
    XDG_CACHE_HOME=/home/terman/.cache \
    XDG_RUNTIME_DIR=/home/terman/.run

COPY --chmod=0755 ${TARGETPLATFORM}/terman-tmux /usr/local/bin/terman-tmux

USER 10001:10001
WORKDIR /home/terman
ENTRYPOINT ["/usr/local/bin/terman-tmux"]
