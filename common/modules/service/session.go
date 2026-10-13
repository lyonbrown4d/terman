package service

import (
	"context"
	"errors"
	"fmt"
	"os"
	"os/exec"
	"strings"
)

type session struct {
	launch  Launch
	command *exec.Cmd
	tree    processTree
	done    chan struct{}
}

func (b *broker) startSession(launch Launch) (int, error) {
	if launch.Name == "" || len(launch.Name) > 256 || strings.ContainsAny(launch.Name, "\x00\r\n") {
		return 0, fmt.Errorf("invalid service session name")
	}
	if launch.PID != 0 {
		return 0, fmt.Errorf("session PID is assigned by the service")
	}
	if launch.Cwd == "" {
		return 0, fmt.Errorf("session working directory is required")
	}
	info, err := os.Stat(launch.Cwd)
	if err != nil {
		return 0, fmt.Errorf("session working directory: %w", err)
	}
	if !info.IsDir() {
		return 0, fmt.Errorf("session working directory is not a directory")
	}
	for _, arg := range launch.Args {
		if strings.ContainsRune(arg, 0) {
			return 0, fmt.Errorf("session argument contains a null character")
		}
	}
	args := append([]string{b.cfg.ServerArgument}, launch.Args...)
	launch.Args = append([]string(nil), launch.Args...)
	command := exec.Command(b.executable, args...)
	command.Dir = launch.Cwd
	tree, err := prepareProcess(command)
	if err != nil {
		return 0, err
	}
	b.mu.Lock()
	defer b.mu.Unlock()
	if b.stopping {
		tree.close()
		return 0, fmt.Errorf("service broker is stopping")
	}
	if len(b.sessions) >= 128 {
		tree.close()
		return 0, fmt.Errorf("service session limit reached")
	}
	if err = command.Start(); err != nil {
		tree.close()
		return 0, fmt.Errorf("start session server: %w", err)
	}
	if err = tree.attach(command.Process); err != nil {
		_ = command.Process.Kill()
		_ = command.Wait()
		tree.close()
		return 0, fmt.Errorf("contain session process: %w", err)
	}
	launch.PID = command.Process.Pid
	child := &session{launch: launch, command: command, tree: tree, done: make(chan struct{})}
	b.sessions[launch.PID] = child
	go func() {
		err := command.Wait()
		tree.close()
		if err != nil {
			b.logger.Debug("session server exited", "name", launch.Name, "pid", launch.PID, "error", err)
		}
		b.mu.Lock()
		delete(b.sessions, launch.PID)
		close(child.done)
		b.mu.Unlock()
	}()
	b.logger.Info("session server started", "name", launch.Name, "pid", launch.PID)
	return launch.PID, nil
}

func (b *broker) stopSession(ctx context.Context, child *session) error {
	select {
	case <-child.done:
		return nil
	default:
	}
	var shutdownErr error
	if b.cfg.Shutdown != nil {
		result := make(chan error, 1)
		go func() { result <- b.cfg.Shutdown(ctx, child.launch) }()
		select {
		case shutdownErr = <-result:
		case <-ctx.Done():
			shutdownErr = ctx.Err()
		}
	} else {
		shutdownErr = child.tree.signal(child.command.Process)
	}
	if shutdownErr == nil {
		select {
		case <-child.done:
			return nil
		case <-ctx.Done():
			shutdownErr = ctx.Err()
		}
	}
	select {
	case <-child.done:
		return nil
	default:
	}
	killErr := child.tree.kill(child.command.Process)
	if errors.Is(killErr, os.ErrProcessDone) {
		killErr = nil
	}
	<-child.done
	if shutdownErr != nil {
		b.logger.Warn("session required forced shutdown", "name", child.launch.Name, "pid", child.launch.PID, "error", shutdownErr)
	}
	return killErr
}
