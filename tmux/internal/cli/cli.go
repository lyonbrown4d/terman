package cli

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/lyonbrown4d/terman/common"
	"github.com/lyonbrown4d/terman/tmux/internal/client"
	"github.com/lyonbrown4d/terman/tmux/internal/ipc"
	"github.com/lyonbrown4d/terman/tmux/internal/platform"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
	"github.com/lyonbrown4d/terman/tmux/internal/server"
	"github.com/lyonbrown4d/terman/tmux/internal/store"
	"github.com/spf13/cobra"
)

func Execute(args []string) error {
	root := &cobra.Command{
		Use: "terman-tmux", Short: "native Windows/Linux terminal multiplexer",
		SilenceUsage: true, SilenceErrors: true, DisableFlagParsing: true,
		RunE: func(_ *cobra.Command, values []string) error { return dispatch(values) },
	}
	root.SetArgs(args)
	return root.Execute()
}

func dispatch(args []string) error {
	if len(args) == 0 {
		return runNew(nil)
	}
	if args[0] == "__server" {
		return runServer(args[1:])
	}
	command := firstCommand(args)
	switch command {
	case "new", "new-session":
		return runNew(args)
	case "attach", "attach-session", "a":
		return runAttach(args)
	case "list-sessions", "ls":
		return listSessions(args)
	case "has-session", "has":
		_, err := targetRecord(args)
		return err
	case "kill-session":
		return simple(args, protocol.Request{Op: "shutdown"})
	case "kill-server":
		return killServer()
	case "rename-session":
		return renameSession(args)
	case "display-message", "display":
		return displayMessage(args)
	case "capture-pane", "capturep":
		return printData(args, protocol.Request{Op: "capture-pane", Window: windowArg(args), Pane: paneArg(args)})
	case "send-keys", "send":
		return sendKeys(args)
	case "send-prefix":
		return simple(args, protocol.Request{Op: "input", Data: "\x02"})
	case "split-window", "splitw":
		return splitPane(args)
	case "swap-pane", "swapp":
		return swapPane(args)
	case "new-window", "neww":
		return newWindow(args)
	case "list-windows", "lsw":
		return listWindows(args)
	case "list-panes", "listp":
		return listPanes(args)
	case "select-pane", "selectp":
		return selectPane(args)
	case "display-panes", "displayp":
		return displayPanes(args)
	case "resize-pane", "resizep":
		return resizePane(args)
	case "select-layout":
		return simple(args, protocol.Request{Op: "select-layout", Window: windowArg(args), Name: firstPositional(args, command)})
	case "next-layout":
		return simple(args, protocol.Request{Op: "select-layout", Window: windowArg(args), Name: "next"})
	case "set-window-option", "setw", "set-option", "set":
		return setOption(args)
	case "refresh-client", "refresh":
		return refresh(args)
	case "select-window", "selectw":
		return simple(args, protocol.Request{Op: "select-window", Window: windowArg(args)})
	case "next-window", "next":
		return simple(args, protocol.Request{Op: "select-window-relative", Direction: "next"})
	case "previous-window", "previous", "prev":
		return simple(args, protocol.Request{Op: "select-window-relative", Direction: "previous"})
	case "last-window", "last":
		return simple(args, protocol.Request{Op: "last-window"})
	case "kill-window", "killw":
		return simple(args, protocol.Request{Op: "kill-window", Window: windowArg(args)})
	case "kill-pane", "killp":
		return simple(args, protocol.Request{Op: "kill-pane", Window: windowArg(args), Pane: paneArg(args)})
	case "rename-window", "renamew":
		return simple(args, protocol.Request{Op: "rename-window", Window: windowArg(args), Name: firstPositional(args, command)})
	case "clear-history":
		return simple(args, protocol.Request{Op: "clear-history", Window: windowArg(args), Pane: paneArg(args)})
	case "set-buffer", "setb", "show-buffer", "showb", "list-buffers", "lsb", "delete-buffer", "deleteb", "paste-buffer", "pasteb":
		return bufferCommand(command, args)
	case "list-clients", "lsc":
		return listClients(args)
	case "detach-client", "detach":
		return simple(args, protocol.Request{Op: "detach"})
	default:
		return fmt.Errorf("unknown command %q", command)
	}
}

func runServer(args []string) error {
	name, endpoint := option(args, "--name", ""), option(args, "--endpoint", "")
	if err := store.ValidateName(name); err != nil {
		return err
	}
	if endpoint == "" {
		return fmt.Errorf("server endpoint is required")
	}
	cols, _ := strconv.Atoi(option(args, "--cols", "80"))
	rows, _ := strconv.Atoi(option(args, "--rows", "24"))
	created, _ := time.Parse(time.RFC3339Nano, option(args, "--created", ""))
	return server.Run(context.Background(), server.Config{
		Name: name, Endpoint: endpoint, Command: option(args, "--command", ""),
		Cols: cols, Rows: rows, Created: created,
	})
}

func runNew(args []string) error {
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
	rec, err := store.Reserve(name, endpoint)
	if err != nil {
		return err
	}
	command := payloadAfter(args, []string{"new", "new-session"}, map[string]bool{"-s": true, "--session-name": true, "-n": true})
	cols, rows := terminalSize()
	executable, err := os.Executable()
	if err != nil {
		_ = store.Remove(name)
		return fmt.Errorf("locate executable: %w", err)
	}
	serverArgs := []string{"__server", "--name", name, "--endpoint", endpoint, "--cols", strconv.Itoa(cols), "--rows", strconv.Itoa(rows), "--created", rec.CreatedAt.Format(time.RFC3339Nano)}
	if command != "" {
		serverArgs = append(serverArgs, "--command", command)
	}
	pid, err := platform.StartServer(executable, serverArgs)
	if err != nil {
		_ = store.Remove(name)
		return err
	}
	rec.PID = pid
	_ = store.Save(rec)
	deadline := time.Now().Add(4 * time.Second)
	for time.Now().Before(deadline) {
		if ipc.Ping(endpoint) {
			if has(args, "-d", "--detached") {
				fmt.Println(name)
				return nil
			}
			return client.Attach(context.Background(), rec)
		}
		time.Sleep(40 * time.Millisecond)
	}
	_ = store.Remove(name)
	return fmt.Errorf("session server did not become ready")
}

func runAttach(args []string) error {
	record, err := targetRecord(args)
	if err != nil {
		return err
	}
	return client.Attach(context.Background(), record)
}

func targetRecord(args []string) (store.Record, error) {
	target := optionAny(args, []string{"-t", "--target-session"}, "")
	session, _, _ := parseTarget(target)
	return client.Find(session)
}

func simple(args []string, request protocol.Request) error {
	record, err := targetRecord(args)
	if err != nil {
		return err
	}
	response, err := client.Call(context.Background(), record, request)
	if err == nil && response.Data != "" {
		fmt.Println(response.Data)
	}
	return err
}

func printData(args []string, request protocol.Request) error {
	record, err := targetRecord(args)
	if err != nil {
		return err
	}
	response, err := client.Call(context.Background(), record, request)
	if err != nil {
		return err
	}
	fmt.Print(response.Data)
	if response.Data != "" && !strings.HasSuffix(response.Data, "\n") {
		fmt.Println()
	}
	return nil
}

func terminalSize() (int, int) {
	cols, rows, err := common.CurrentTerminalSize()
	if err != nil || cols < 2 || rows < 2 {
		return 80, 24
	}
	return int(cols), int(rows)
}

func printJSON(value any) error {
	data, err := json.MarshalIndent(value, "", "  ")
	if err != nil {
		return err
	}
	fmt.Println(string(data))
	return nil
}
