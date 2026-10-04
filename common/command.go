package common

import (
	"context"
	"errors"
	"io"
	"os"
	"os/exec"
	"time"
)

// DefaultCommandTimeout is the timeout used by command probes.
const DefaultCommandTimeout = 8 * time.Second

var terminalEnvAllowlist = [...]string{
	"TERM",
	"COLORTERM",
	"LC_ALL",
	"LANG",
	"LC_CTYPE",
	"TERM_PROGRAM",
	"TERM_PROGRAM_VERSION",
}

// CommandStatusWithTimeout runs a silent command and waits at most timeout.
// It returns nil, nil when the timeout expires. A non-zero exit is a status,
// not an execution error.
func CommandStatusWithTimeout(command string, args []string, timeout time.Duration) (*os.ProcessState, error) {
	return CommandStatusWithTimeoutContext(context.Background(), command, args, timeout)
}

// CommandStatusWithTimeoutContext is CommandStatusWithTimeout with parent cancellation.
func CommandStatusWithTimeoutContext(
	parent context.Context,
	command string,
	args []string,
	timeout time.Duration,
) (*os.ProcessState, error) {
	ctx, cancel := context.WithTimeout(parent, timeout)
	defer cancel()

	cmd := exec.CommandContext(ctx, command, args...)
	cmd.Stdout = io.Discard
	cmd.Stderr = io.Discard
	if err := cmd.Start(); err != nil {
		return nil, err
	}

	err := cmd.Wait()
	if errors.Is(ctx.Err(), context.DeadlineExceeded) {
		return nil, nil
	}
	if err == nil {
		return cmd.ProcessState, nil
	}
	var exitError *exec.ExitError
	if errors.As(err, &exitError) {
		return exitError.ProcessState, nil
	}
	return nil, err
}

// WhichBinary resolves name using the current PATH.
func WhichBinary(name string) (string, bool) {
	path, err := exec.LookPath(name)
	return path, err == nil
}

// PassthroughEnv returns only terminal-related environment variables.
func PassthroughEnv() []string {
	vars := make([]string, 0, len(terminalEnvAllowlist))
	for _, name := range terminalEnvAllowlist {
		if value, ok := os.LookupEnv(name); ok {
			vars = append(vars, name+"="+value)
		}
	}
	return vars
}

// TerminalEnv returns the terminal environment with a safe TERM default.
func TerminalEnv() []string {
	vars := PassthroughEnv()
	if _, ok := os.LookupEnv("TERM"); !ok {
		vars = append(vars, "TERM=xterm-256color")
	}
	return vars
}
