package session

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/lyonbrown4d/terman/common"
	"github.com/lyonbrown4d/terman/screen/internal/proto"
)

const Version = "0.2.0"

func (o *Owner) control(request proto.Request) proto.Response {
	switch request.Type {
	case "ping":
		return accepted("pong")
	case "input":
		if err := o.writeWindow(request.Target, request.Data); err != nil {
			return rejected(err)
		}
		return accepted("")
	case "resize":
		o.cols, o.rows = max(request.Cols, 20), max(request.Rows, 5)
		o.resizeWindows()
		o.broadcast()
		return accepted("")
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
	case "title", "aka":
		return o.setTitle(request)
	case "sessionname":
		return o.renameSession(args)
	case "info", "dinfo":
		return accepted(o.info())
	case "version":
		return accepted(common.LocalizedMessage(
			"builtin-screen-control-version",
			map[string]string{"version": Version},
		))
	case "help", "commands":
		return accepted(common.LocalizedMessage("builtin-screen-control-help", nil))
	case "stuff":
		if len(args) == 0 {
			return rejected(fmt.Errorf("stuff requires text"))
		}
		if err := o.writeWindow(request.Target, []byte(strings.Join(args, " "))); err != nil {
			return rejected(err)
		}
	case "width", "height":
		return o.setDimensions(command, args)
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
		if err := o.resizeRegion(args); err != nil {
			return rejected(err)
		}
	case "fit", "redisplay":
		o.resizeWindows()
		o.broadcast()
	case "scrollback", "defscrollback":
		return o.setScrollback(args)
	case "copy":
		o.storeRegister(".", request.Data)
	case "register", "readreg", "readbuf", "writebuf", "removebuf", "paste",
		"bufferfile", "pastefile":
		return o.bufferCommand(command, args, request.Target)
	case "hardcopy", "hardcopydir", "hardcopy_append":
		return o.captureCommand(command, args, request.Target)
	case "log", "logfile", "logtstamp", "deflog":
		return o.logCommand(command, args, request.Target)
	case "number":
		return accepted(strconv.Itoa(o.active))
	case "lastmsg":
		return accepted(o.lastMessage)
	default:
		return rejected(fmt.Errorf("%s", common.LocalizedMessage(
			"builtin-screen-control-command-unsupported",
			map[string]string{"command": command},
		)))
	}
	return accepted("")
}

func (o *Owner) setDimensions(command string, args []string) proto.Response {
	if len(args) == 0 {
		value := o.cols
		if command == "height" {
			value = o.rows
		}
		return accepted(strconv.Itoa(value))
	}
	value, err := strconv.Atoi(args[0])
	if err != nil {
		return rejected(fmt.Errorf("invalid %s: %w", command, err))
	}
	if command == "height" {
		o.rows = max(value, 5)
	} else {
		o.cols = max(value, 20)
		if len(args) > 1 {
			rows, parseErr := strconv.Atoi(args[1])
			if parseErr != nil {
				return rejected(fmt.Errorf("invalid height: %w", parseErr))
			}
			o.rows = max(rows, 5)
		}
	}
	o.resizeWindows()
	o.broadcast()
	return accepted("")
}

func (o *Owner) writeWindow(target string, data []byte) error {
	index := o.targetWindow(target)
	if index < 0 {
		return fmt.Errorf("window not found")
	}
	return o.windows[index].Write(data)
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
