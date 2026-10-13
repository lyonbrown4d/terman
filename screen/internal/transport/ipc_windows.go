//go:build windows

package transport

import (
	"context"
	"net"

	"github.com/Microsoft/go-winio"
	"github.com/lyonbrown4d/terman/common/modules/service"
	"github.com/lyonbrown4d/terman/screen/internal/paths"
)

func Endpoint(name string) (string, error) {
	identity, err := service.UserIdentity()
	if err != nil {
		return "", err
	}
	return `\\.\pipe\terman-screen-` + identity + "-" + paths.Key(name), nil
}

func Listen(endpoint string) (net.Listener, error) {
	descriptor, err := service.PipeSecurityDescriptor()
	if err != nil {
		return nil, err
	}
	return winio.ListenPipe(endpoint, &winio.PipeConfig{
		SecurityDescriptor: descriptor,
		InputBufferSize:    65536, OutputBufferSize: 65536,
	})
}

func Dial(ctx context.Context, endpoint string) (net.Conn, error) {
	return winio.DialPipeContext(ctx, endpoint)
}

func Cleanup(string) error { return nil }
