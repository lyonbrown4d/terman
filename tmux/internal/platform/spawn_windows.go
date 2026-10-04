//go:build windows

package platform

import (
	"os/exec"
	"syscall"
)

type sysProcAttr = syscall.SysProcAttr

func newProcessAttributes() *sysProcAttr {
	const detachedProcess = 0x00000008
	return &syscall.SysProcAttr{
		CreationFlags: syscall.CREATE_NEW_PROCESS_GROUP | detachedProcess,
		HideWindow:    true,
	}
}

func applyProcessAttributes(cmd *exec.Cmd, attr *sysProcAttr) {
	cmd.SysProcAttr = attr
}
