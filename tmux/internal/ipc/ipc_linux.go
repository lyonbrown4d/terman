//go:build linux

package ipc

import (
	"context"
	"crypto/sha256"
	"fmt"
	"net"
	"os"
	"path/filepath"
)

func Endpoint(name string) string {
	sum := sha256.Sum256([]byte(name))
	base := os.Getenv("XDG_RUNTIME_DIR")
	if base == "" {
		base = os.TempDir()
	}
	return filepath.Join(base, fmt.Sprintf("terman-tmux-%x.sock", sum[:16]))
}

func Listen(endpoint string) (net.Listener, error) {
	if err := os.Remove(endpoint); err != nil && !os.IsNotExist(err) {
		return nil, err
	}
	listener, err := net.Listen("unix", endpoint)
	if err == nil {
		_ = os.Chmod(endpoint, 0o600)
	}
	return listener, err
}

func Dial(ctx context.Context, endpoint string) (net.Conn, error) {
	var d net.Dialer
	return d.DialContext(ctx, "unix", endpoint)
}

func Cleanup(endpoint string) error {
	err := os.Remove(endpoint)
	if os.IsNotExist(err) {
		return nil
	}
	return err
}
