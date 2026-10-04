//go:build linux

package transport

import (
	"context"
	"errors"
	"io/fs"
	"net"
	"os"
	"path/filepath"

	"github.com/lyonbrown4d/terman/screen/internal/paths"
)

func Endpoint(name string) (string, error) {
	root, err := paths.Root()
	if err != nil {
		return "", err
	}
	return filepath.Join(root, paths.Key(name)+".sock"), nil
}

func Listen(endpoint string) (net.Listener, error) {
	_ = Cleanup(endpoint)
	listener, err := net.Listen("unix", endpoint)
	if err != nil {
		return nil, err
	}
	if err := os.Chmod(endpoint, 0o600); err != nil {
		_ = listener.Close()
		return nil, err
	}
	return listener, nil
}

func Dial(ctx context.Context, endpoint string) (net.Conn, error) {
	return (&net.Dialer{}).DialContext(ctx, "unix", endpoint)
}

func Cleanup(endpoint string) error {
	err := os.Remove(endpoint)
	if errors.Is(err, fs.ErrNotExist) {
		return nil
	}
	return err
}
