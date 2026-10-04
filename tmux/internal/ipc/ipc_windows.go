//go:build windows

package ipc

import (
	"context"
	"crypto/sha256"
	"fmt"
	"net"

	"github.com/Microsoft/go-winio"
)

func Endpoint(name string) string {
	sum := sha256.Sum256([]byte(name))
	return fmt.Sprintf("\\\\.\\pipe\\terman-tmux-%x", sum[:16])
}

func Listen(endpoint string) (net.Listener, error) {
	return winio.ListenPipe(endpoint, &winio.PipeConfig{
		InputBufferSize: 64 * 1024, OutputBufferSize: 64 * 1024,
	})
}

func Dial(ctx context.Context, endpoint string) (net.Conn, error) {
	return winio.DialPipeContext(ctx, endpoint)
}

func Cleanup(string) error { return nil }
