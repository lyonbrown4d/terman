package ipc

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"net"
	"time"

	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
)

func Call(ctx context.Context, endpoint string, req protocol.Request) (protocol.Response, error) {
	conn, err := Dial(ctx, endpoint)
	if err != nil {
		return protocol.Response{}, fmt.Errorf("connect session: %w", err)
	}
	defer conn.Close()
	if deadline, ok := ctx.Deadline(); ok {
		_ = conn.SetDeadline(deadline)
	}
	if err := json.NewEncoder(conn).Encode(req); err != nil {
		return protocol.Response{}, fmt.Errorf("send request: %w", err)
	}
	var resp protocol.Response
	if err := json.NewDecoder(bufio.NewReader(conn)).Decode(&resp); err != nil {
		return protocol.Response{}, fmt.Errorf("read response: %w", err)
	}
	if !resp.OK {
		return resp, fmt.Errorf("%s", resp.Error)
	}
	return resp, nil
}

func Ping(endpoint string) bool {
	ctx, cancel := context.WithTimeout(context.Background(), 250*time.Millisecond)
	defer cancel()
	resp, err := Call(ctx, endpoint, protocol.Request{Op: "ping"})
	return err == nil && resp.OK
}

func Encode(conn net.Conn, value any) error {
	return json.NewEncoder(conn).Encode(value)
}
