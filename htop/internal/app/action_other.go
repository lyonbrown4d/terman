//go:build !linux && !windows

package app

import (
	"errors"
	"fmt"
	"os"
)

func signalProcess(pid int32, signal ProcessSignal) error {
	if signal != SignalTerm && signal != SignalKill {
		return fmt.Errorf("signal %s is unsupported on this platform", signal)
	}
	process, err := os.FindProcess(int(pid))
	if err != nil {
		return err
	}
	return process.Kill()
}

func setProcessPriority(pid int32, nice int32) error {
	return errors.New("priority changes are unsupported on this platform")
}
