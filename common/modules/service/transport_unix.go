//go:build !windows

package service

import (
	"context"
	"errors"
	"fmt"
	"net"
	"os"
	"path/filepath"
	"strconv"
	"syscall"

	"golang.org/x/sys/unix"
)

// UserIdentity returns the current effective UID.
func UserIdentity() (string, error) {
	return strconv.Itoa(os.Geteuid()), nil
}

// PipeSecurityDescriptor is empty on Unix, where directory and socket modes
// protect the native Unix-domain endpoint.
func PipeSecurityDescriptor() (string, error) { _, err := UserIdentity(); return "", err }

func endpoint(tool, identity string) (string, error) {
	path := filepath.Join(os.TempDir(), "terman-service-"+identity, tool+".sock")
	if len(path) >= 104 {
		return "", fmt.Errorf("service Unix socket path is too long")
	}
	return path, nil
}

type lockedListener struct {
	net.Listener
	lock *os.File
}

func (l *lockedListener) Close() error {
	err := l.Listener.Close()
	_ = unix.Flock(int(l.lock.Fd()), unix.LOCK_UN)
	return errors.Join(err, l.lock.Close())
}

func listenEndpoint(address string) (net.Listener, error) {
	dir := filepath.Dir(address)
	if err := os.Mkdir(dir, 0700); err != nil && !errors.Is(err, os.ErrExist) {
		return nil, err
	}
	info, err := os.Lstat(dir)
	if err != nil {
		return nil, err
	}
	stat, ok := info.Sys().(*syscall.Stat_t)
	if !info.IsDir() || !ok || int(stat.Uid) != os.Geteuid() || info.Mode().Perm() != 0700 {
		return nil, fmt.Errorf("service socket directory must be owned by the current user with mode 0700")
	}
	lock, err := os.OpenFile(address+".lock", os.O_CREATE|os.O_RDWR, 0600)
	if err != nil {
		return nil, err
	}
	if err = unix.Flock(int(lock.Fd()), unix.LOCK_EX|unix.LOCK_NB); err != nil {
		lock.Close()
		return nil, fmt.Errorf("service broker is already running: %w", err)
	}
	fail := func(err error) (net.Listener, error) {
		_ = unix.Flock(int(lock.Fd()), unix.LOCK_UN)
		lock.Close()
		return nil, err
	}
	if info, err = os.Lstat(address); err == nil {
		stat, ok = info.Sys().(*syscall.Stat_t)
		if info.Mode()&os.ModeSocket == 0 || !ok || int(stat.Uid) != os.Geteuid() {
			return fail(fmt.Errorf("unsafe existing service socket"))
		}
		if err = os.Remove(address); err != nil {
			return fail(err)
		}
	} else if !errors.Is(err, os.ErrNotExist) {
		return fail(err)
	}
	listener, err := net.ListenUnix("unix", &net.UnixAddr{Name: address, Net: "unix"})
	if err != nil {
		return fail(err)
	}
	if err = os.Chmod(address, 0600); err != nil {
		listener.Close()
		return fail(err)
	}
	return &lockedListener{Listener: listener, lock: lock}, nil
}
func dialEndpoint(ctx context.Context, address string) (net.Conn, error) {
	for _, path := range []string{filepath.Dir(address), address} {
		info, err := os.Lstat(path)
		if err != nil {
			return nil, err
		}
		stat, ok := info.Sys().(*syscall.Stat_t)
		expected := os.FileMode(0600)
		if path != address {
			expected = 0700
		}
		if !ok || int(stat.Uid) != os.Geteuid() || info.Mode().Perm() != expected || info.Mode()&os.ModeSymlink != 0 {
			return nil, fmt.Errorf("service socket path has unsafe ownership or permissions")
		}
	}
	var dialer net.Dialer
	conn, err := dialer.DialContext(ctx, "unix", address)
	if err != nil {
		return nil, err
	}
	if err = validateUnixPeer(conn); err != nil {
		conn.Close()
		return nil, err
	}
	return conn, nil
}
func endpointUnavailable(err error) bool {
	return errors.Is(err, os.ErrNotExist) || errors.Is(err, syscall.ECONNREFUSED)
}
func restoreUserProfile() error { _, err := UserIdentity(); return err }
func configurePlatform(cfg *nativeConfig, username, passwordEnv string) error {
	if username != "" || passwordEnv != "" {
		return fmt.Errorf("account flags are only supported on Windows")
	}
	if err := requireRegularUser(); err != nil {
		return err
	}
	cfg.Option["UserService"] = true
	return validatePlatform()
}

func requireRegularUser() error {
	if os.Geteuid() == 0 {
		return serviceError("service-root-rejected", nil)
	}
	return nil
}
