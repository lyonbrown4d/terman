package platform

import (
	"fmt"
	"os"
	"os/exec"
)

func start(executable string, args []string, attr *sysProcAttr) (int, error) {
	null, err := os.OpenFile(os.DevNull, os.O_RDWR, 0)
	if err != nil {
		return 0, fmt.Errorf("open null device: %w", err)
	}
	defer null.Close()
	cmd := exec.Command(executable, args...)
	cmd.Stdin, cmd.Stdout, cmd.Stderr = null, null, null
	applyProcessAttributes(cmd, attr)
	if err := cmd.Start(); err != nil {
		return 0, fmt.Errorf("start session server: %w", err)
	}
	pid := cmd.Process.Pid
	if err := cmd.Process.Release(); err != nil {
		return 0, fmt.Errorf("release session server: %w", err)
	}
	return pid, nil
}

func StartServer(executable string, args []string) (int, error) {
	return start(executable, args, newProcessAttributes())
}
