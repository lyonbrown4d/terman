package client

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/lyonbrown4d/terman/screen/internal/proto"
	"github.com/lyonbrown4d/terman/screen/internal/transport"
)

func Request(ctx context.Context, endpoint string, request proto.Request) (proto.Response, error) {
	requestCtx, cancel := context.WithTimeout(ctx, 2*time.Second)
	defer cancel()
	conn, err := transport.Dial(requestCtx, endpoint)
	if err != nil {
		return proto.Response{}, err
	}
	defer conn.Close()
	deadline, _ := requestCtx.Deadline()
	if err := conn.SetDeadline(deadline); err != nil {
		return proto.Response{}, err
	}
	stop := context.AfterFunc(requestCtx, func() { _ = conn.Close() })
	defer stop()
	if err := json.NewEncoder(conn).Encode(request); err != nil {
		return proto.Response{}, err
	}
	var response proto.Response
	if err := json.NewDecoder(conn).Decode(&response); err != nil {
		if requestCtx.Err() != nil {
			return proto.Response{}, requestCtx.Err()
		}
		return proto.Response{}, err
	}
	if response.Error != "" {
		return response, fmt.Errorf("%s", response.Error)
	}
	return response, nil
}

func Ping(ctx context.Context, endpoint string) bool {
	response, err := Request(ctx, endpoint, proto.Request{Type: "ping"})
	return err == nil && response.Message == "pong"
}
