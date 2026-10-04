package cli

import (
	"context"
	"fmt"
	"strconv"
	"strings"

	"github.com/lyonbrown4d/terman/tmux/internal/client"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
)

func newWindow(args []string) error {
	name := optionAny(args, []string{"-n", "--window-name"}, "")
	command := payloadAfter(
		args,
		[]string{"new-window", "neww"},
		map[string]bool{
			"-t": true, "--target-session": true,
			"-n": true, "--window-name": true,
		},
	)
	return simple(args, protocol.Request{
		Op: "new-window", Name: name, Data: command,
	})
}

func listWindows(args []string) error {
	record, err := targetRecord(args)
	if err != nil {
		return err
	}
	resp, err := client.Call(
		context.Background(),
		record,
		protocol.Request{Op: "list-windows"},
	)
	if err != nil {
		return err
	}
	if has(args, "--json") {
		return printJSON(struct {
			SchemaVersion int                   `json:"schema_version"`
			Session       string                `json:"session"`
			Windows       []protocol.WindowInfo `json:"windows"`
		}{protocol.SchemaVersion, record.Name, resp.Windows})
	}
	for _, value := range resp.Windows {
		marker := "-"
		if value.Active {
			marker = "*"
		}
		fmt.Printf(
			"%s:%d: %s%s (%d panes) [%s]\n",
			record.Name,
			value.Index,
			value.Name,
			marker,
			value.PaneCount,
			value.Layout,
		)
	}
	return nil
}

func listPanes(args []string) error {
	record, err := targetRecord(args)
	if err != nil {
		return err
	}
	window := windowArg(args)
	resp, err := client.Call(
		context.Background(),
		record,
		protocol.Request{Op: "list-panes", Window: window},
	)
	if err != nil {
		return err
	}
	if has(args, "--json") {
		index := 0
		if window != nil {
			index = *window
		} else if len(resp.Panes) > 0 {
			index = resp.Panes[0].Window
		}
		return printJSON(struct {
			SchemaVersion int                 `json:"schema_version"`
			Session       string              `json:"session"`
			WindowIndex   int                 `json:"window_index"`
			Panes         []protocol.PaneInfo `json:"panes"`
		}{protocol.SchemaVersion, record.Name, index, resp.Panes})
	}
	for _, value := range resp.Panes {
		marker := "-"
		if value.Active {
			marker = "*"
		}
		fmt.Printf(
			"%s:%d.%d: %dx%d%s\n",
			record.Name,
			value.Window,
			value.Index,
			value.Width,
			value.Height,
			marker,
		)
	}
	return nil
}

func splitPane(args []string) error {
	command := payloadAfter(
		args,
		[]string{"split-window", "splitw"},
		map[string]bool{
			"-t": true, "--target-session": true, "-c": true,
		},
	)
	return simple(args, protocol.Request{
		Op: "split-pane", Window: windowArg(args),
		Horizontal: has(args, "-h", "--horizontal"), Data: command,
	})
}

func selectPane(args []string) error {
	return simple(args, protocol.Request{
		Op: "select-pane", Window: windowArg(args),
		Pane: paneArg(args), Direction: directionArg(args),
	})
}

func swapPane(args []string) error {
	direction := ""
	if has(args, "-U") {
		direction = "previous"
	} else if has(args, "-D") {
		direction = "next"
	}
	source := paneFrom(optionAny(
		args,
		[]string{"-s", "--source-pane"},
		"",
	))
	return simple(args, protocol.Request{
		Op: "swap-pane", Window: windowArg(args),
		Pane: paneArg(args), SourcePane: source, Direction: direction,
	})
}

func resizePane(args []string) error {
	delta := 1
	for _, arg := range args {
		if value, err := strconv.Atoi(arg); err == nil && value > 0 {
			delta = value
		}
	}
	if has(args, "-Z", "--zoom") {
		return simple(args, protocol.Request{
			Op: "zoom-pane", Window: windowArg(args), Pane: paneArg(args),
		})
	}
	width, _ := strconv.Atoi(optionAny(
		args, []string{"-x", "--width"}, "0",
	))
	height, _ := strconv.Atoi(optionAny(
		args, []string{"-y", "--height"}, "0",
	))
	return simple(args, protocol.Request{
		Op: "resize-pane", Window: windowArg(args), Pane: paneArg(args),
		Direction: directionArg(args), Delta: delta,
		Width: width, Height: height,
	})
}

func displayPanes(args []string) error {
	return simple(args, protocol.Request{
		Op: "display-panes", Window: windowArg(args),
	})
}

func refresh(args []string) error {
	cols, rows := terminalSize()
	clientID := optionAny(
		args,
		[]string{"-c", "--target-client"},
		"",
	)
	return simple(args, protocol.Request{
		Op: "resize-client", ClientID: clientID,
		Width: cols, Height: rows,
	})
}

func detachClient(args []string) error {
	clientID := optionAny(
		args,
		[]string{"-c", "--target-client"},
		"",
	)
	return simple(args, protocol.Request{
		Op:       "detach-client",
		ClientID: clientID,
		All:      has(args, "-a", "--all") || strings.TrimSpace(clientID) == "",
	})
}
