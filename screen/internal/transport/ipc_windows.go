//go:build windows

package transport

import (
	"context"
	"net"

	"github.com/Microsoft/go-winio"
	"github.com/lyonbrown4d/terman/screen/internal/paths"
)

func Endpoint(name string) (string, error) {
	return `\\.\pipe\terman-screen-` + paths.Key(name), nil
}

func Listen(endpoint string) (net.Listener, error) {
	return winio.ListenPipe(endpoint, &winio.PipeConfig{
		InputBufferSize: 65536, OutputBufferSize: 65536,
	})
}

func Dial(ctx context.Context, endpoint string) (net.Conn, error) {
	return winio.DialPipeContext(ctx, endpoint)
}

func Cleanup(string) error { return nil }
