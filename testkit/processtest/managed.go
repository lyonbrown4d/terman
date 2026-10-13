package processtest

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"sync"
)

const processOutputLimit = 1 << 20

// Process owns a started test process and its captured stdout and stderr.
type Process struct {
	command *exec.Cmd
	output  processOutput
	done    chan struct{}
	err     error
}

// Start launches a real process. The caller should stop it before test cleanup.
func Start(
	ctx context.Context,
	directory string,
	executable string,
	env []string,
	args ...string,
) (*Process, error) {
	process := &Process{
		command: exec.CommandContext(ctx, executable, args...),
		done:    make(chan struct{}),
	}
	process.command.Dir = directory
	process.command.Env = Environment(os.Environ(), env...)
	process.command.Stdout = &process.output
	process.command.Stderr = &process.output
	if err := process.command.Start(); err != nil {
		return nil, fmt.Errorf("start %s: %w", executable, err)
	}
	go func() {
		process.err = process.command.Wait()
		close(process.done)
	}()
	return process, nil
}

func (process *Process) PID() int {
	return process.command.Process.Pid
}

func (process *Process) Output() string {
	process.output.mu.Lock()
	defer process.output.mu.Unlock()
	return process.output.buffer.String()
}

func (process *Process) Wait(ctx context.Context) error {
	select {
	case <-process.done:
		return process.err
	case <-ctx.Done():
		return ctx.Err()
	}
}

func (process *Process) Kill() error {
	err := process.command.Process.Kill()
	if errors.Is(err, os.ErrProcessDone) {
		return nil
	}
	return err
}

type processOutput struct {
	mu     sync.Mutex
	buffer bytes.Buffer
}

func (output *processOutput) Write(data []byte) (int, error) {
	output.mu.Lock()
	defer output.mu.Unlock()
	count := len(data)
	remaining := processOutputLimit - output.buffer.Len()
	if remaining > 0 {
		_, _ = output.buffer.Write(data[:min(len(data), remaining)])
	}
	return count, nil
}
