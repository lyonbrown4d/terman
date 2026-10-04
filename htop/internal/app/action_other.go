//go:build !linux && !windows

package app

import (
	"errors"
	"os"
)

func signalProcess(pid int32, force bool) error {
	process, err := os.FindProcess(int(pid))
	if err != nil {
		return err
	}
	return process.Kill()
}

func setProcessPriority(pid int32, nice int32) error {
	return errors.New("priority changes are unsupported on this platform")
}
