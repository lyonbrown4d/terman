//go:build linux

package app

import "syscall"

func signalProcess(pid int32, force bool) error {
	signal := syscall.SIGTERM
	if force {
		signal = syscall.SIGKILL
	}
	return syscall.Kill(int(pid), signal)
}

func setProcessPriority(pid int32, nice int32) error {
	return syscall.Setpriority(syscall.PRIO_PROCESS, int(pid), int(nice))
}
