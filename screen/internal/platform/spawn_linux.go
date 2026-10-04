//go:build linux

package platform

import (
	"os/exec"
	"syscall"
)

func StartDetached(executable string, args []string) error {
	cmd := exec.Command(executable, args...)
	cmd.SysProcAttr = &syscall.SysProcAttr{Setsid: true}
	return cmd.Start()
}
