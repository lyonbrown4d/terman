package client

import (
	"testing"

	"github.com/gdamore/tcell/v3"
)

func TestGNUDefaultPrefixBindings(t *testing.T) {
	tests := []struct {
		name    string
		key     *tcell.EventKey
		command string
		local   string
		detach  bool
	}{
		{name: "detach", key: runeKey("d"), detach: true},
		{name: "create", key: runeKey("c"), command: "screen"},
		{name: "next-space", key: runeKey(" "), command: "next"},
		{name: "next-n", key: runeKey("n"), command: "next"},
		{name: "previous", key: runeKey("p"), command: "prev"},
		{name: "kill", key: runeKey("k"), command: "kill"},
		{name: "quit", key: runeKey("\\"), command: "quit"},
		{name: "title", key: runeKey("A"), local: "title"},
		{name: "help", key: runeKey("?"), local: "help"},
		{name: "command", key: runeKey(":"), local: "command"},
		{name: "copy", key: runeKey("["), local: "copy"},
		{name: "paste", key: runeKey("]"), command: "paste"},
		{name: "hardcopy", key: runeKey("h"), command: "hardcopy"},
		{name: "log", key: runeKey("H"), command: "log"},
		{name: "split", key: runeKey("S"), command: "split"},
		{name: "focus", key: tcell.NewEventKey(tcell.KeyTAB, "", tcell.ModNone), command: "focus"},
		{name: "only", key: runeKey("Q"), command: "only"},
		{name: "remove", key: runeKey("X"), command: "remove"},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			action := prefixAction(test.key)
			if action.command != test.command || action.local != test.local ||
				action.detach != test.detach {
				t.Fatalf(
					"got command=%q local=%q detach=%t",
					action.command,
					action.local,
					action.detach,
				)
			}
		})
	}
}

func runeKey(value string) *tcell.EventKey {
	return tcell.NewEventKey(tcell.KeyRune, value, tcell.ModNone)
}
