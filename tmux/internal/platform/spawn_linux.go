//go:build linux

package platform

import (
	"os/exec"
	"syscall"
)

type sysProcAttr = syscall.SysProcAttr

func newProcessAttributes() *sysProcAttr {
	return &syscall.SysProcAttr{Setsid: true}
}

func applyProcessAttributes(cmd *exec.Cmd, attr *sysProcAttr) {
	cmd.SysProcAttr = attr
}
