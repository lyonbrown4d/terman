//go:build windows

package service

import (
	"fmt"
	"net"

	"golang.org/x/sys/windows"
)

func validatePipePeer(conn net.Conn) error {
	pipe, ok := conn.(interface{ Fd() uintptr })
	if !ok {
		return fmt.Errorf("service pipe does not expose its native handle")
	}
	var pid uint32
	if err := windows.GetNamedPipeServerProcessId(windows.Handle(pipe.Fd()), &pid); err != nil {
		return err
	}
	process, err := windows.OpenProcess(windows.PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
	if err != nil {
		return err
	}
	defer windows.CloseHandle(process)
	var token windows.Token
	if err = windows.OpenProcessToken(process, windows.TOKEN_QUERY, &token); err != nil {
		return err
	}
	defer token.Close()
	account, err := token.GetTokenUser()
	if err != nil {
		return err
	}
	current, err := UserIdentity()
	if err != nil {
		return err
	}
	if account.User.Sid.String() != current {
		return serviceError("service-user-mismatch", nil)
	}
	return nil
}
