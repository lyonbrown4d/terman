package cli

import (
	"context"
	"fmt"
	"strings"
	"unicode/utf8"

	"github.com/lyonbrown4d/terman/tmux/internal/client"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
)

func sendKeys(args []string) error {
	command := firstCommand(args)
	values := positionalValues(args, command)
	literal := has(args, "-l", "--literal")
	var output strings.Builder
	for _, value := range values {
		if literal {
			output.WriteString(value)
			continue
		}
		switch strings.ToLower(value) {
		case "enter":
			output.WriteByte('\r')
		case "space":
			output.WriteByte(' ')
		case "tab":
			output.WriteByte('\t')
		case "escape", "esc":
			output.WriteByte(0x1b)
		case "bspace", "backspace":
			output.WriteByte(0x7f)
		case "up":
			output.WriteString("\x1b[A")
		case "down":
			output.WriteString("\x1b[B")
		case "right":
			output.WriteString("\x1b[C")
		case "left":
			output.WriteString("\x1b[D")
		default:
			if len(value) == 3 && strings.HasPrefix(strings.ToLower(value), "c-") {
				r, _ := utf8.DecodeRuneInString(strings.ToLower(value[2:]))
				if r >= 'a' && r <= 'z' {
					output.WriteByte(byte(r - 'a' + 1))
					continue
				}
			}
			output.WriteString(value)
		}
	}
	return simple(args, protocol.Request{Op: "input", Window: windowArg(args), Pane: paneArg(args), Data: output.String()})
}

func setOption(args []string) error {
	values := positionalValues(args, firstCommand(args))
	if len(values) == 0 || values[0] != "synchronize-panes" {
		return fmt.Errorf("only synchronize-panes is supported")
	}
	var enabled *bool
	if len(values) > 1 && !strings.EqualFold(values[1], "toggle") {
		value := values[1] == "on" || values[1] == "1" || strings.EqualFold(values[1], "true")
		enabled = &value
	}
	return simple(args, protocol.Request{Op: "set-synchronize-panes", Window: windowArg(args), Enabled: enabled})
}

func bufferCommand(command string, args []string) error {
	name := optionAny(args, []string{"-b", "--buffer-name"}, "")
	request := protocol.Request{Name: name, Window: windowArg(args), Pane: paneArg(args)}
	switch command {
	case "set-buffer", "setb":
		request.Op = "set-buffer"
		request.Data = strings.Join(positionalValues(args, command), " ")
	case "show-buffer", "showb":
		request.Op = "get-buffer"
	case "list-buffers", "lsb":
		record, err := targetRecord(args)
		if err != nil {
			return err
		}
		resp, err := client.Call(context.Background(), record, protocol.Request{Op: "list-buffers"})
		if err != nil {
			return err
		}
		if has(args, "--json") {
			return printJSON(struct {
				SchemaVersion int                   `json:"schema_version"`
				Session       string                `json:"session"`
				Buffers       []protocol.BufferInfo `json:"buffers"`
			}{protocol.SchemaVersion, record.Name, resp.Buffers})
		}
		for _, value := range resp.Buffers {
			fmt.Printf("%s: %d bytes: %s\n", value.Name, value.Bytes, value.Preview)
		}
		return nil
	case "delete-buffer", "deleteb":
		request.Op = "delete-buffer"
	case "paste-buffer", "pasteb":
		request.Op = "paste-buffer"
	}
	if request.Op == "get-buffer" {
		return printData(args, request)
	}
	return simple(args, request)
}
