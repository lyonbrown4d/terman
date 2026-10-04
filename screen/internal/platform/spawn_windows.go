//go:build windows

package platform

import (
	"os/exec"
	"syscall"
)

func StartDetached(executable string, args []string) error {
	cmd := exec.Command(executable, args...)
	cmd.SysProcAttr = &syscall.SysProcAttr{
		HideWindow: true, CreationFlags: 0x00000008 | 0x00000200,
	}
	return cmd.Start()
}
