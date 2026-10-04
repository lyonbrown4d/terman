package app

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io/fs"
	"os"
	"strconv"
	"time"

	"github.com/cenkalti/backoff/v7"
	"github.com/lyonbrown4d/terman/common"
	"github.com/lyonbrown4d/terman/screen/internal/client"
	"github.com/lyonbrown4d/terman/screen/internal/platform"
	"github.com/lyonbrown4d/terman/screen/internal/proto"
	"github.com/lyonbrown4d/terman/screen/internal/session"
	"github.com/lyonbrown4d/terman/screen/internal/store"
	"github.com/lyonbrown4d/terman/screen/internal/transport"
)

func execute(ctx context.Context, args Args, config PersistentConfig) error {
	switch {
	case args.Server:
		if args.Session == "" || args.Endpoint == "" {
			return fmt.Errorf("internal server requires session and endpoint")
		}
		serverConfig := session.DefaultConfig(
			args.Session,
			args.Endpoint,
			args.Command,
			config.Terminal.Cols,
			config.Terminal.Rows,
			config.sessionSettings(),
		)
		return session.Serve(ctx, serverConfig)
	case args.Wipe:
		count, err := wipe(ctx)
		if err == nil {
			fmt.Println(common.LocalizedMessage(
				"builtin-screen-wipe-complete",
				map[string]string{"count": strconv.Itoa(count)},
			))
		}
		return err
	case args.List:
		return list(ctx, args.JSON)
	case args.Execute != "":
		return executeControl(ctx, args)
	case args.Resume || args.Multi:
		return attachExisting(ctx, args)
	case args.ResumeCreate:
		record, err := findLive(ctx, args.AttachName)
		if err == nil {
			return client.Attach(ctx, record.Endpoint, "resume", args.DetachExisting)
		}
		if !errors.Is(err, fs.ErrNotExist) {
			return err
		}
		args.Session = args.AttachName
	}
	return createSession(ctx, args, config)
}

func executeControl(ctx context.Context, args Args) error {
	record, err := findLive(ctx, args.Session)
	if err != nil {
		return err
	}
	response, err := client.Request(ctx, record.Endpoint, proto.Request{
		Type: "command", Command: args.Execute,
		Args: args.ExecuteArgs, Target: args.Window,
	})
	if response.Message != "" {
		fmt.Println(response.Message)
	}
	return err
}

func attachExisting(ctx context.Context, args Args) error {
	record, err := findLive(ctx, args.AttachName)
	if err != nil {
		return err
	}
	mode := "resume"
	if args.Multi {
		mode = "multi"
	}
	return client.Attach(ctx, record.Endpoint, mode, args.DetachExisting)
}

func createSession(ctx context.Context, args Args, config PersistentConfig) error {
	if args.Session == "" {
		args.Session = fmt.Sprintf("screen-%d", os.Getpid())
	}
	if err := validateName(args.Session); err != nil {
		return err
	}
	endpoint, err := transport.Endpoint(args.Session)
	if err != nil {
		return err
	}
	if record, findErr := findLive(ctx, args.Session); findErr == nil {
		return fmt.Errorf("session %q already exists at %s", args.Session, record.Endpoint)
	}
	if err := spawn(args, endpoint, config); err != nil {
		return err
	}
	if err := waitReady(ctx, endpoint); err != nil {
		return err
	}
	if args.Detached {
		return nil
	}
	return client.Attach(ctx, endpoint, "resume", false)
}

func spawn(args Args, endpoint string, config PersistentConfig) error {
	executable, err := os.Executable()
	if err != nil {
		return err
	}
	serverArgs := []string{
		"--__screen-server", "--__endpoint-name", endpoint,
		"-S", args.Session,
		"--cols", strconv.Itoa(config.Terminal.Cols),
		"--rows", strconv.Itoa(config.Terminal.Rows),
		"--term", config.Terminal.Name,
		"--scrollback", strconv.Itoa(config.Terminal.Scrollback),
		"--login-shell=" + strconv.FormatBool(config.Defaults.LoginShell),
		"--hardcopydir", config.Defaults.HardcopyDir,
		"--hardcopy-append=" + strconv.FormatBool(config.Defaults.HardcopyAppend),
		"--logfile", config.Logging.File,
		"--deflog=" + strconv.FormatBool(config.Logging.Enabled),
		"--logtstamp=" + strconv.FormatBool(config.Logging.Timestamp),
		"--logtstamp-after", config.Logging.TimestampAfter.String(),
		"--logtstamp-format", config.Logging.TimestampFormat,
	}
	if args.Command != "" {
		serverArgs = append(serverArgs, "--command", args.Command)
	}
	return platform.StartDetached(executable, serverArgs)
}

func waitReady(ctx context.Context, endpoint string) error {
	notReady := errors.New("screen server is not ready")
	_, err := backoff.Retry(ctx, func() (bool, error) {
		if client.Ping(ctx, endpoint) {
			return true, nil
		}
		return false, notReady
	},
		backoff.WithBackOff(backoff.NewConstantBackOff(25*time.Millisecond)),
		backoff.WithMaxElapsedTime(3*time.Second),
	)
	if err == nil {
		return nil
	}
	if ctx.Err() != nil {
		return ctx.Err()
	}
	return fmt.Errorf("screen server startup timed out")
}

func findLive(ctx context.Context, name string) (store.Record, error) {
	record, err := store.Find(name)
	if err != nil {
		return record, err
	}
	if !client.Ping(ctx, record.Endpoint) {
		_ = store.Delete(record.Name)
		return store.Record{}, fs.ErrNotExist
	}
	return record, nil
}

func wipe(ctx context.Context) (int, error) {
	records, err := store.List()
	if err != nil {
		return 0, err
	}
	count := 0
	for _, record := range records {
		if !client.Ping(ctx, record.Endpoint) {
			if err := store.Delete(record.Name); err != nil {
				return count, err
			}
			_ = transport.Cleanup(record.Endpoint)
			count++
		}
	}
	return count, nil
}

func list(ctx context.Context, asJSON bool) error {
	records, err := store.List()
	if err != nil {
		return err
	}
	live := records[:0]
	for _, record := range records {
		if client.Ping(ctx, record.Endpoint) {
			live = append(live, record)
		}
	}
	if asJSON {
		return json.NewEncoder(os.Stdout).Encode(live)
	}
	if len(live) == 0 {
		fmt.Println(common.LocalizedMessage("builtin-screen-no-sessions", nil))
		return nil
	}
	fmt.Println(common.LocalizedMessage("builtin-screen-session-list-header", nil))
	for _, record := range live {
		fmt.Printf(
			"  %s pid=%d cwd=%s command=%s\n",
			record.Name,
			record.PID,
			record.Cwd,
			record.Command,
		)
	}
	return nil
}

func validateName(name string) error {
	if name == "" || len(name) > 64 {
		return fmt.Errorf("invalid session name")
	}
	for _, char := range name {
		valid := char == '-' || char == '_' || char == '.' ||
			char >= 'a' && char <= 'z' || char >= 'A' && char <= 'Z' ||
			char >= '0' && char <= '9'
		if !valid {
			return fmt.Errorf("invalid session name %q", name)
		}
	}
	return nil
}
