//go:build linux

package transport

import (
	"context"
	"errors"
	"fmt"
	"net"
	"os"
	"path/filepath"
	"sync"
	"syscall"
	"time"

	"github.com/gofrs/flock"
	"github.com/lyonbrown4d/terman/common/modules/service"
	"github.com/lyonbrown4d/terman/screen/internal/paths"
)

func Endpoint(name string) (string, error) {
	identity, err := service.UserIdentity()
	if err != nil {
		return "", err
	}
	base := os.Getenv("XDG_RUNTIME_DIR")
	if base == "" {
		base = os.TempDir()
	}
	endpoint := filepath.Join(base, "terman-screen-"+identity, paths.Key(name)+".sock")
	if len(endpoint) >= 104 {
		return "", fmt.Errorf("session Unix socket path is too long")
	}
	return endpoint, nil
}

type sessionListener struct {
	net.Listener
	lock *flock.Flock
	once sync.Once
	err  error
}

func (l *sessionListener) Close() error {
	l.once.Do(func() { l.err = errors.Join(l.Listener.Close(), l.lock.Unlock()) })
	return l.err
}

func Listen(endpoint string) (net.Listener, error) {
	dir := filepath.Dir(endpoint)
	if err := os.Mkdir(dir, 0700); err != nil && !errors.Is(err, os.ErrExist) {
		return nil, err
	}
	if err := validateSocketDirectory(dir); err != nil {
		return nil, err
	}
	lock := flock.New(endpoint + ".lock")
	locked, err := lock.TryLock()
	if err != nil {
		return nil, err
	}
	if !locked {
		return nil, fmt.Errorf("session socket is already in use")
	}
	fail := func(err error) (net.Listener, error) { _ = lock.Unlock(); return nil, err }
	if err := removeStaleSocket(endpoint); err != nil {
		return fail(err)
	}
	listener, err := net.Listen("unix", endpoint)
	if err != nil {
		return fail(err)
	}
	if err = os.Chmod(endpoint, 0600); err != nil {
		_ = listener.Close()
		return fail(err)
	}
	return &sessionListener{Listener: listener, lock: lock}, nil
}

func Dial(ctx context.Context, endpoint string) (net.Conn, error) {
	return (&net.Dialer{}).DialContext(ctx, "unix", endpoint)
}

func Cleanup(endpoint string) error {
	if _, err := os.Lstat(endpoint); errors.Is(err, os.ErrNotExist) {
		return nil
	} else if err != nil {
		return err
	}
	if err := validateSocketDirectory(filepath.Dir(endpoint)); err != nil {
		return err
	}
	lock := flock.New(endpoint + ".lock")
	locked, err := lock.TryLock()
	if err != nil {
		return err
	}
	if !locked {
		return fmt.Errorf("session socket is still in use")
	}
	defer lock.Unlock()
	return removeStaleSocket(endpoint)
}

func validateSocketDirectory(dir string) error {
	info, err := os.Lstat(dir)
	if err != nil {
		return err
	}
	stat, ok := info.Sys().(*syscall.Stat_t)
	if !info.IsDir() || !ok || int(stat.Uid) != os.Geteuid() || info.Mode().Perm() != 0700 {
		return fmt.Errorf("session socket directory must be owned by the current user with mode 0700")
	}
	return nil
}

func removeStaleSocket(endpoint string) error {
	info, err := os.Lstat(endpoint)
	if errors.Is(err, os.ErrNotExist) {
		return nil
	}
	if err != nil {
		return err
	}
	stat, ok := info.Sys().(*syscall.Stat_t)
	if info.Mode()&os.ModeSocket == 0 || !ok || int(stat.Uid) != os.Geteuid() {
		return fmt.Errorf("refusing to remove unsafe session socket path")
	}
	conn, err := net.DialTimeout("unix", endpoint, 100*time.Millisecond)
	if err == nil {
		conn.Close()
		return fmt.Errorf("session endpoint is already listening")
	}
	if !errors.Is(err, syscall.ECONNREFUSED) && !errors.Is(err, os.ErrNotExist) {
		return fmt.Errorf("check existing session socket: %w", err)
	}
	err = os.Remove(endpoint)
	if errors.Is(err, os.ErrNotExist) {
		return nil
	}
	return err
}
