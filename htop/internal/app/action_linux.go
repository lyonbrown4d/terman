//go:build linux

package app

import "syscall"

func signalProcess(pid int32, signal ProcessSignal) error {
	value := syscall.SIGTERM
	switch signal {
	case SignalKill:
		value = syscall.SIGKILL
	case SignalInterrupt:
		value = syscall.SIGINT
	case SignalHangup:
		value = syscall.SIGHUP
	case SignalStop:
		value = syscall.SIGSTOP
	case SignalContinue:
		value = syscall.SIGCONT
	}
	return syscall.Kill(int(pid), value)
}

func setProcessPriority(pid int32, nice int32) error {
	return syscall.Setpriority(syscall.PRIO_PROCESS, int(pid), int(nice))
}
