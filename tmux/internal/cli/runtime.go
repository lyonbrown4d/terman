package cli

import (
	"context"
	"errors"
	"fmt"
	"os"
	"strconv"
	"time"

	"github.com/cenkalti/backoff/v7"
	"github.com/lyonbrown4d/terman/common/modules/service"
	"github.com/lyonbrown4d/terman/tmux/internal/client"
	"github.com/lyonbrown4d/terman/tmux/internal/ipc"
	"github.com/lyonbrown4d/terman/tmux/internal/platform"
	"github.com/lyonbrown4d/terman/tmux/internal/server"
	"github.com/lyonbrown4d/terman/tmux/internal/store"
)

func runServer(ctx context.Context, args []string) error {
	name := option(args, "--name", "")
	endpoint := option(args, "--endpoint", "")
	if err := store.ValidateName(name); err != nil {
		return err
	}
	if endpoint == "" {
		return fmt.Errorf("server endpoint is required")
	}
	cols, _ := strconv.Atoi(option(args, "--cols", "80"))
	rows, _ := strconv.Atoi(option(args, "--rows", "24"))
	created, _ := time.Parse(
		time.RFC3339Nano,
		option(args, "--created", ""),
	)
	return server.Run(
		ctx,
		server.Launch{
			Name: name, Endpoint: endpoint,
			Command: option(args, "--command", ""),
			Cols:    cols, Rows: rows, Created: created,
		},
		settings,
	)
}

func runNew(ctx context.Context, args []string) error {
	name := optionAny(args, []string{"-s", "--session-name"}, "")
	if name == "" {
		records, err := client.LiveRecords(true)
		if err != nil {
			return err
		}
		used := make(map[string]bool)
		for _, rec := range records {
			used[rec.Name] = true
		}
		for i := 0; ; i++ {
			name = strconv.Itoa(i)
			if !used[name] {
				break
			}
		}
	}
	if err := store.ValidateName(name); err != nil {
		return err
	}
	endpoint := ipc.Endpoint(name)
	if endpoint == "" {
		return fmt.Errorf("session user identity is unavailable")
	}
	rec, err := store.Reserve(name, endpoint)
	if err != nil {
		return err
	}
	command := payloadAfter(
		args,
		[]string{"new", "new-session"},
		map[string]bool{
			"-s": true, "--session-name": true, "-n": true,
		},
	)
	cols, rows := terminalSize()
	executable, err := os.Executable()
	if err != nil {
		_ = store.Remove(name)
		return fmt.Errorf("locate executable: %w", err)
	}
	serverArgs := []string{
		"__server", "--name", name, "--endpoint", endpoint,
		"--cols", strconv.Itoa(cols), "--rows", strconv.Itoa(rows),
		"--created", rec.CreatedAt.Format(time.RFC3339Nano),
	}
	serverArgs = append(serverArgs, configFlagArgs(settings)...)
	if command != "" {
		serverArgs = append(serverArgs, "--command", command)
	}
	cwd, err := os.Getwd()
	if err != nil {
		_ = store.Remove(name)
		return fmt.Errorf("session working directory: %w", err)
	}
	pid, used, err := service.TryStart(ctx, ServiceConfig(), service.Launch{
		Name: name, Cwd: cwd, Args: serverArgs[1:],
	})
	if err == nil && !used {
		pid, err = platform.StartServer(executable, serverArgs)
	}
	if err != nil {
		_ = store.Remove(name)
		return err
	}
	rec.PID = pid
	_ = store.Save(rec)
	startupCtx, cancel := context.WithTimeout(
		ctx, settings.StartupTimeout,
	)
	notReady := errors.New("session server is not ready")
	_, err = backoff.Retry(startupCtx, func() (bool, error) {
		if ipc.Ping(endpoint) {
			return true, nil
		}
		return false, notReady
	},
		backoff.WithBackOff(backoff.NewConstantBackOff(40*time.Millisecond)),
		backoff.WithMaxElapsedTime(0),
	)
	cancel()
	if err != nil {
		_ = store.Remove(name)
		return fmt.Errorf("session server did not become ready")
	}
	if has(args, "-d", "--detached") {
		fmt.Println(name)
		return nil
	}
	return client.Attach(ctx, rec)
}
