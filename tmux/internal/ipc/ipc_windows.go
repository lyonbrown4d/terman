//go:build windows

package ipc

import (
	"context"
	"crypto/sha256"
	"fmt"
	"net"

	"github.com/Microsoft/go-winio"
	"github.com/lyonbrown4d/terman/common/modules/service"
)

func Endpoint(name string) string {
	identity, err := service.UserIdentity()
	if err != nil {
		return ""
	}
	sum := sha256.Sum256([]byte(name))
	return fmt.Sprintf("\\\\.\\pipe\\terman-tmux-%s-%x", identity, sum[:16])
}

func Listen(endpoint string) (net.Listener, error) {
	descriptor, err := service.PipeSecurityDescriptor()
	if err != nil {
		return nil, err
	}
	return winio.ListenPipe(endpoint, &winio.PipeConfig{
		SecurityDescriptor: descriptor,
		InputBufferSize:    64 * 1024, OutputBufferSize: 64 * 1024,
	})
}

func Dial(ctx context.Context, endpoint string) (net.Conn, error) {
	return winio.DialPipeContext(ctx, endpoint)
}

func Cleanup(string) error { return nil }
