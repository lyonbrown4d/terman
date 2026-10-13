package cli

import (
	"context"
	"fmt"

	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
)

func dispatch(ctx context.Context, args []string) error {
	if len(args) == 0 {
		return runNew(ctx, nil)
	}
	if args[0] == "__server" {
		return runServer(ctx, args[1:])
	}
	command := firstCommand(args)
	switch command {
	case "new", "new-session":
		return runNew(ctx, args)
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
		return printData(args, protocol.Request{
			Op: "capture-pane", Window: windowArg(args), Pane: paneArg(args),
		})
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
		return simple(args, protocol.Request{
			Op: "select-layout", Window: windowArg(args),
			Name: firstPositional(args, command),
		})
	case "next-layout":
		return simple(args, protocol.Request{
			Op: "select-layout", Window: windowArg(args), Name: "next",
		})
	case "set-window-option", "setw", "set-option", "set":
		return setOption(args)
	case "refresh-client", "refresh":
		return refresh(args)
	case "select-window", "selectw":
		return simple(args, protocol.Request{
			Op: "select-window", Window: windowArg(args),
		})
	case "next-window", "next":
		return simple(args, protocol.Request{
			Op: "select-window-relative", Direction: "next",
		})
	case "previous-window", "previous", "prev":
		return simple(args, protocol.Request{
			Op: "select-window-relative", Direction: "previous",
		})
	case "last-window", "last":
		return simple(args, protocol.Request{Op: "last-window"})
	case "kill-window", "killw":
		return simple(args, protocol.Request{
			Op: "kill-window", Window: windowArg(args),
		})
	case "kill-pane", "killp":
		return simple(args, protocol.Request{
			Op: "kill-pane", Window: windowArg(args), Pane: paneArg(args),
		})
	case "rename-window", "renamew":
		return simple(args, protocol.Request{
			Op: "rename-window", Window: windowArg(args),
			Name: firstPositional(args, command),
		})
	case "clear-history":
		return simple(args, protocol.Request{
			Op: "clear-history", Window: windowArg(args), Pane: paneArg(args),
		})
	case "set-buffer", "setb", "show-buffer", "showb",
		"list-buffers", "lsb", "delete-buffer", "deleteb",
		"paste-buffer", "pasteb":
		return bufferCommand(command, args)
	case "list-clients", "lsc":
		return listClients(args)
	case "detach-client", "detach":
		return detachClient(args)
	default:
		return fmt.Errorf("unknown command %q", command)
	}
}
