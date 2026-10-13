//go:build linux

package service

import (
	"fmt"
	"net"
	"os"
	"syscall"

	native "github.com/kardianos/service"
	"golang.org/x/sys/unix"
)

func validatePlatform() error {
	if native.Platform() != "linux-systemd" {
		return serviceError("service-unsupported-platform", nil)
	}
	return nil
}

func validateUnixPeer(conn net.Conn) error {
	socket, ok := conn.(syscall.Conn)
	if !ok {
		return fmt.Errorf("service socket does not expose its native handle")
	}
	raw, err := socket.SyscallConn()
	if err != nil {
		return err
	}
	var peerErr error
	err = raw.Control(func(fd uintptr) {
		credentials, err := unix.GetsockoptUcred(int(fd), unix.SOL_SOCKET, unix.SO_PEERCRED)
		if err != nil {
			peerErr = err
			return
		}
		if int(credentials.Uid) != os.Geteuid() {
			peerErr = serviceError("service-user-mismatch", nil)
		}
	})
	if err != nil {
		return err
	}
	return peerErr
}
