package session

import (
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/lyonbrown4d/terman/common"
	"github.com/lyonbrown4d/terman/screen/internal/proto"
	"github.com/lyonbrown4d/terman/screen/internal/store"
)

const Version = "0.1.0"

func (o *Owner) control(request proto.Request) proto.Response {
	switch request.Type {
	case "ping":
		return proto.Response{Type: "accepted", Message: "pong"}
	case "input":
		window := o.targetWindow(request.Target)
		if window < 0 {
			return rejected(fmt.Errorf("window not found"))
		}
		if err := o.windows[window].Write(request.Data); err != nil {
			return rejected(err)
		}
		return proto.Response{Type: "accepted"}
	case "resize":
		o.cols, o.rows = max(request.Cols, 20), max(request.Rows, 5)
		o.resizeWindows()
		o.broadcast()
		return proto.Response{Type: "accepted"}
	case "detach":
		o.detachClient(request.ClientID)
		return proto.Response{Type: "detached"}
	case "command":
		return o.command(request)
	default:
		return rejected(fmt.Errorf("unsupported request %q", request.Type))
	}
}

func (o *Owner) command(request proto.Request) proto.Response {
	command := strings.ToLower(strings.TrimSpace(request.Command))
	args := request.Args
	switch command {
	case "windows", "windowlist":
		return accepted(o.windowsText())
	case "screen":
		if err := o.newWindow(strings.Join(args, " ")); err != nil {
			return rejected(err)
		}
	case "select":
		if err := o.selectWindow(first(args)); err != nil {
			return rejected(err)
		}
	case "next":
		o.navigate(1)
	case "prev", "previous":
		o.navigate(-1)
	case "last", "other":
		o.last, o.active = o.active, o.last
		o.regions[o.focused].windowID = o.windows[o.active].ID
		o.broadcast()
	case "kill":
		o.closeWindow(o.targetWindow(request.Target))
	case "quit":
		return proto.Response{Type: "accepted", Exit: true}
	case "title":
		if len(args) == 0 {
			return rejected(fmt.Errorf("title requires text"))
		}
		index := o.targetWindow(request.Target)
		if index < 0 {
			return rejected(fmt.Errorf("window not found"))
		}
		o.windows[index].Title = strings.Join(args, " ")
		o.broadcast()
	case "sessionname":
		if len(args) == 0 {
			return accepted(o.config.Name)
		}
		if err := validateName(args[0]); err != nil {
			return rejected(err)
		}
		old := o.config.Name
		o.config.Name = args[0]
		if err := store.Rename(old, o.record()); err != nil {
			o.config.Name = old
			return rejected(err)
		}
		o.broadcast()
	case "info":
		return accepted(o.info())
	case "version":
		return accepted(common.LocalizedMessage("builtin-screen-control-version", map[string]string{"version": Version}))
	case "stuff":
		if len(args) == 0 {
			return rejected(fmt.Errorf("stuff requires text"))
		}
		index := o.targetWindow(request.Target)
		if index < 0 {
			return rejected(fmt.Errorf("window not found"))
		}
		if err := o.windows[index].Write([]byte(strings.Join(args, " "))); err != nil {
			return rejected(err)
		}
	case "width":
		if len(args) == 0 {
			return accepted(strconv.Itoa(o.cols))
		}
		cols, err := strconv.Atoi(args[0])
		if err != nil {
			return rejected(err)
		}
		rows := o.rows
		if len(args) > 1 {
			rows, err = strconv.Atoi(args[1])
			if err != nil {
				return rejected(err)
			}
		}
		o.cols, o.rows = max(cols, 20), max(rows, 5)
		o.resizeWindows()
		o.broadcast()
	case "split":
		o.split(len(args) > 0 && (args[0] == "-v" || args[0] == "vertical"))
	case "focus":
		index := -1
		if len(args) > 0 {
			index, _ = strconv.Atoi(args[0])
		}
		o.focus(index)
	case "remove":
		o.removeRegion()
	case "only":
		o.onlyRegion()
	case "resize":
		o.resizeWindows()
		o.broadcast()
	default:
		return rejected(fmt.Errorf("%s", common.LocalizedMessage(
			"builtin-screen-control-command-unsupported", map[string]string{"command": command})))
	}
	return proto.Response{Type: "accepted"}
}

func (o *Owner) targetWindow(selector string) int {
	if selector == "" {
		return o.active
	}
	index, err := strconv.Atoi(selector)
	if err == nil && index >= 0 && index < len(o.windows) {
		return index
	}
	for index, window := range o.windows {
		if window.Title == selector {
			return index
		}
	}
	return -1
}

func (o *Owner) windowsText() string {
	var lines []string
	for index, window := range o.windows {
		marker := "-"
		if index == o.active {
			marker = "*"
		}
		lines = append(lines, fmt.Sprintf("%d%s %s", index, marker, window.Title))
	}
	return strings.Join(lines, "\n")
}

func (o *Owner) info() string {
	var bytes int64
	for _, window := range o.windows {
		bytes += window.Bytes
	}
	return common.LocalizedMessage("builtin-screen-control-info", map[string]string{
		"session_name": o.config.Name, "replay_bytes": strconv.FormatInt(bytes, 10),
		"attach_clients": strconv.Itoa(len(o.clients)), "cols": strconv.Itoa(o.cols),
		"rows": strconv.Itoa(o.rows), "scrollback_lines": "1000",
	})
}

func (o *Owner) record() store.Record {
	command := o.config.Command
	if command == "" {
		command = common.DefaultShell()
	}
	return store.Record{
		Name: o.config.Name, PID: os.Getpid(), Endpoint: o.config.Endpoint,
		Cwd: o.cwd, Command: command, Started: time.Now(),
	}
}

func accepted(message string) proto.Response {
	return proto.Response{Type: "accepted", Message: message}
}

func rejected(err error) proto.Response {
	return proto.Response{Type: "rejected", Error: err.Error()}
}

func first(values []string) string {
	if len(values) == 0 {
		return ""
	}
	return values[0]
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
