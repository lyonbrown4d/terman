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

	"github.com/lyonbrown4d/terman/common"
	"github.com/lyonbrown4d/terman/screen/internal/client"
	"github.com/lyonbrown4d/terman/screen/internal/platform"
	"github.com/lyonbrown4d/terman/screen/internal/proto"
	"github.com/lyonbrown4d/terman/screen/internal/session"
	"github.com/lyonbrown4d/terman/screen/internal/store"
	"github.com/lyonbrown4d/terman/screen/internal/transport"
)

func Run(ctx context.Context, argv []string) error {
	args, err := Parse(argv)
	if err != nil {
		return err
	}
	if args.Help {
		fmt.Print(helpText())
		return nil
	}
	if args.Version {
		fmt.Println("terman-screen " + session.Version)
		return nil
	}
	if args.Server {
		if args.Session == "" || args.Endpoint == "" {
			return fmt.Errorf("internal server requires session and endpoint")
		}
		return session.Serve(ctx, session.DefaultConfig(
			args.Session, args.Endpoint, args.Command, args.Cols, args.Rows, args.LoginShell))
	}
	if args.Wipe {
		count, wipeErr := wipe(ctx)
		if wipeErr == nil {
			fmt.Println(common.LocalizedMessage("builtin-screen-wipe-complete",
				map[string]string{"count": strconv.Itoa(count)}))
		}
		return wipeErr
	}
	if args.List {
		return list(ctx, args.JSON)
	}
	if args.Execute != "" {
		record, findErr := findLive(ctx, args.Session)
		if findErr != nil {
			return findErr
		}
		response, requestErr := client.Request(ctx, record.Endpoint, proto.Request{
			Type: "command", Command: args.Execute, Args: args.ExecuteArgs, Target: args.Window,
		})
		if response.Message != "" {
			fmt.Println(response.Message)
		}
		return requestErr
	}
	if args.Resume || args.Multi {
		record, findErr := findLive(ctx, args.AttachName)
		if findErr != nil {
			return findErr
		}
		mode := "resume"
		if args.Multi {
			mode = "multi"
		}
		return client.Attach(ctx, record.Endpoint, mode, args.DetachExisting)
	}
	if args.ResumeCreate {
		record, findErr := findLive(ctx, args.AttachName)
		if findErr == nil {
			return client.Attach(ctx, record.Endpoint, "resume", args.DetachExisting)
		}
		if !errors.Is(findErr, fs.ErrNotExist) {
			return findErr
		}
		args.Session = args.AttachName
	}
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
	if err := spawn(args, endpoint); err != nil {
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

func spawn(args Args, endpoint string) error {
	executable, err := os.Executable()
	if err != nil {
		return err
	}
	serverArgs := []string{
		"--__screen-server", "--__endpoint-name", endpoint,
		"-S", args.Session, "--cols", strconv.Itoa(args.Cols), "--rows", strconv.Itoa(args.Rows),
	}
	if args.Command != "" {
		serverArgs = append(serverArgs, "--command", args.Command)
	}
	if args.LoginShell {
		serverArgs = append(serverArgs, "--login-shell")
	}
	return platform.StartDetached(executable, serverArgs)
}

func waitReady(ctx context.Context, endpoint string) error {
	timer := time.NewTimer(3 * time.Second)
	defer timer.Stop()
	ticker := time.NewTicker(25 * time.Millisecond)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-timer.C:
			return fmt.Errorf("screen server startup timed out")
		case <-ticker.C:
			if client.Ping(ctx, endpoint) {
				return nil
			}
		}
	}
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
		fmt.Printf("  %s pid=%d cwd=%s command=%s\n",
			record.Name, record.PID, record.Cwd, record.Command)
	}
	return nil
}

func validateName(name string) error {
	if name == "" || len(name) > 64 {
		return fmt.Errorf("invalid session name")
	}
	for _, char := range name {
		if !(char == '-' || char == '_' || char == '.' ||
			char >= 'a' && char <= 'z' || char >= 'A' && char <= 'Z' ||
			char >= '0' && char <= '9') {
			return fmt.Errorf("invalid session name %q", name)
		}
	}
	return nil
}

func helpText() string {
	return "terman-screen: native cross-platform terminal sessions\n\n" +
		"Usage: terman-screen [-S name] [-d|-r|-R|-x] [command]\n" +
		"       terman-screen --list [--json] | --wipe\n" +
		"       terman-screen -S name [-p window] -X command [args...]\n\n" +
		common.LocalizedMessage("builtin-screen-cli-about", nil) + "\n"
}
