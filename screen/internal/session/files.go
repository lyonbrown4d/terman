package session

import (
	"fmt"
	"os"
	"strconv"
	"strings"

	"github.com/lyonbrown4d/terman/common"
	"github.com/lyonbrown4d/terman/screen/internal/proto"
	"github.com/lyonbrown4d/terman/screen/internal/store"
)

func (o *Owner) setTitle(request proto.Request) proto.Response {
	index := o.targetWindow(request.Target)
	if index < 0 {
		return rejected(fmt.Errorf("window not found"))
	}
	if len(request.Args) == 0 {
		return accepted(o.windows[index].Title)
	}
	o.windows[index].Title = strings.Join(request.Args, " ")
	o.broadcast()
	return accepted("")
}

func (o *Owner) renameSession(args []string) proto.Response {
	if len(args) == 0 {
		return accepted(o.config.Name)
	}
	if err := validateName(args[0]); err != nil {
		return rejected(err)
	}
	oldName := o.config.Name
	record := o.record()
	record.Name = args[0]
	if err := store.Rename(oldName, record); err != nil {
		return rejected(err)
	}
	o.config.Name = args[0]
	o.broadcast()
	return accepted("")
}

func (o *Owner) setScrollback(args []string) proto.Response {
	if len(args) != 1 {
		return rejected(fmt.Errorf("scrollback requires a line count"))
	}
	lines, err := strconv.Atoi(args[0])
	if err != nil || lines < 0 || lines > 100000 {
		return rejected(fmt.Errorf("invalid scrollback line count %q", args[0]))
	}
	o.scrollback = lines
	for _, window := range o.windows {
		window.SetScrollbackSize(lines)
	}
	o.broadcast()
	return accepted("")
}

func (o *Owner) info() string {
	var bytes int64
	for _, window := range o.windows {
		bytes += window.Bytes
	}
	return common.LocalizedMessage("builtin-screen-control-info", map[string]string{
		"session_name":     o.config.Name,
		"replay_bytes":     strconv.FormatInt(bytes, 10),
		"attach_clients":   strconv.Itoa(len(o.clients)),
		"cols":             strconv.Itoa(o.cols),
		"rows":             strconv.Itoa(o.rows),
		"scrollback_lines": strconv.Itoa(o.scrollback),
	})
}

func (o *Owner) record() store.Record {
	command := o.config.Command
	if command == "" {
		command = common.DefaultShell()
	}
	return store.Record{
		Name: o.config.Name, PID: os.Getpid(), Endpoint: o.config.Endpoint,
		Cwd: o.cwd, Command: command, Started: o.started,
	}
}

func parseSwitch(args []string) (bool, error) {
	if len(args) != 1 {
		return false, fmt.Errorf("expected on or off")
	}
	switch strings.ToLower(args[0]) {
	case "on", "yes", "true", "1":
		return true, nil
	case "off", "no", "false", "0":
		return false, nil
	default:
		return false, fmt.Errorf("expected on or off, got %q", args[0])
	}
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
