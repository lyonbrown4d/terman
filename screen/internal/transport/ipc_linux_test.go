//go:build linux

package transport

import (
	"net"
	"os"
	"path/filepath"
	"testing"
)

func TestListenPreservesExistingPaths(t *testing.T) {
	for _, kind := range []string{"regular-file", "symlink", "live-socket", "stale-socket"} {
		t.Run(kind, func(t *testing.T) {
			dir := t.TempDir()
			path := filepath.Join(dir, "session.sock")
			switch kind {
			case "regular-file":
				if err := os.WriteFile(path, []byte("preserve"), 0600); err != nil {
					t.Fatal(err)
				}
			case "symlink":
				target := filepath.Join(dir, "target")
				if err := os.WriteFile(target, []byte("preserve"), 0600); err != nil {
					t.Fatal(err)
				}
				if err := os.Symlink(target, path); err != nil {
					t.Fatal(err)
				}
			default:
				listener, err := net.ListenUnix("unix", &net.UnixAddr{Name: path, Net: "unix"})
				if err != nil {
					t.Fatal(err)
				}
				listener.SetUnlinkOnClose(false)
				if kind == "stale-socket" {
					_ = listener.Close()
				} else {
					defer listener.Close()
				}
			}
			listener, err := Listen(path)
			if kind == "stale-socket" {
				if err != nil {
					t.Fatalf("recover stale owned socket: %v", err)
				}
				defer listener.Close()
				info, err := os.Stat(path)
				if err != nil || info.Mode().Perm() != 0600 {
					t.Fatalf("socket mode: %v, %v", info, err)
				}
			} else {
				if err == nil {
					listener.Close()
					t.Fatalf("replaced existing %s", kind)
				}
				if _, err := os.Lstat(path); err != nil {
					t.Fatalf("existing path removed: %v", err)
				}
			}
		})
	}
}

func TestListenRejectsUnsafeDirectory(t *testing.T) {
	for _, kind := range []string{"shared-mode", "symlink"} {
		t.Run(kind, func(t *testing.T) {
			base := t.TempDir()
			dir := filepath.Join(base, "private")
			if kind == "symlink" {
				if err := os.Symlink(base, dir); err != nil {
					t.Fatal(err)
				}
			} else {
				if err := os.Mkdir(dir, 0700); err != nil {
					t.Fatal(err)
				}
				if err := os.Chmod(dir, 0777); err != nil {
					t.Fatal(err)
				}
			}
			listener, err := Listen(filepath.Join(dir, "session.sock"))
			if err == nil {
				listener.Close()
				t.Fatalf("accepted unsafe directory %s", kind)
			}
		})
	}
}

func TestCleanupPreservesLiveSocket(t *testing.T) {
	path := filepath.Join(t.TempDir(), "session.sock")
	listener, err := Listen(path)
	if err != nil {
		t.Fatal(err)
	}
	defer listener.Close()
	if err := Cleanup(path); err == nil {
		t.Fatal("cleanup removed live session socket")
	}
	if _, err := os.Stat(path); err != nil {
		t.Fatalf("live socket removed: %v", err)
	}
}
