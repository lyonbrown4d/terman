//go:build windows

package app

import (
	"fmt"
	"os"

	"golang.org/x/sys/windows"
)

func signalProcess(pid int32, signal ProcessSignal) error {
	if signal != SignalTerm && signal != SignalKill {
		return fmt.Errorf("signal %s is unsupported on Windows", signal)
	}
	process, err := os.FindProcess(int(pid))
	if err != nil {
		return err
	}
	return process.Kill()
}

func setProcessPriority(pid int32, nice int32) error {
	handle, err := windows.OpenProcess(
		windows.PROCESS_SET_INFORMATION|windows.PROCESS_QUERY_LIMITED_INFORMATION,
		false,
		uint32(pid),
	)
	if err != nil {
		return err
	}
	defer windows.CloseHandle(handle)
	class := uint32(windows.NORMAL_PRIORITY_CLASS)
	switch {
	case nice <= -10:
		class = windows.HIGH_PRIORITY_CLASS
	case nice <= -1:
		class = windows.ABOVE_NORMAL_PRIORITY_CLASS
	case nice >= 10:
		class = windows.IDLE_PRIORITY_CLASS
	case nice >= 1:
		class = windows.BELOW_NORMAL_PRIORITY_CLASS
	}
	return windows.SetPriorityClass(handle, class)
}
