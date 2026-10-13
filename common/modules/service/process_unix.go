//go:build !windows

package service

import (
	"errors"
	"os"
	"os/exec"
	"syscall"
)

type processTree struct{ pid *int }

func prepareProcess(command *exec.Cmd) (processTree, error) {
	command.SysProcAttr = &syscall.SysProcAttr{Setpgid: true}
	return processTree{pid: new(int)}, nil
}
func (tree processTree) attach(process *os.Process) error { *tree.pid = process.Pid; return nil }
func (processTree) signal(process *os.Process) error {
	return syscall.Kill(-process.Pid, syscall.SIGTERM)
}
func (processTree) kill(process *os.Process) error {
	err := syscall.Kill(-process.Pid, syscall.SIGKILL)
	if errors.Is(err, syscall.ESRCH) {
		return nil
	}
	return err
}
func (tree processTree) close() {
	if tree.pid != nil && *tree.pid > 0 {
		_ = syscall.Kill(-*tree.pid, syscall.SIGKILL)
	}
}
