package client

import (
	"unicode/utf8"

	"github.com/gdamore/tcell/v3"
)

type keyAction struct {
	data    []byte
	command string
	args    []string
	local   string
	detach  bool
}

func decodeKey(event *tcell.EventKey, prefix *bool) keyAction {
	if !*prefix && isCtrl(event, 'a') {
		*prefix = true
		return keyAction{}
	}
	if *prefix {
		*prefix = false
		return prefixAction(event)
	}
	return keyAction{data: keyBytes(event)}
}

func prefixAction(event *tcell.EventKey) keyAction {
	switch event.Key() {
	case tcell.KeyCtrlA:
		return keyAction{data: []byte{1}}
	case tcell.KeyCtrlC:
		return keyAction{command: "screen"}
	case tcell.KeyCtrlD:
		return keyAction{detach: true}
	case tcell.KeyCtrlN:
		return keyAction{command: "next"}
	case tcell.KeyCtrlP:
		return keyAction{command: "prev"}
	case tcell.KeyEsc:
		return keyAction{local: "copy"}
	}
	char := eventRune(event)
	switch char {
	case 'a':
		return keyAction{data: []byte{1}}
	case 'c':
		return keyAction{command: "screen"}
	case 'd':
		return keyAction{detach: true}
	case 'n', ' ':
		return keyAction{command: "next"}
	case 'p':
		return keyAction{command: "prev"}
	case 'k':
		return keyAction{command: "kill"}
	case '\\':
		return keyAction{command: "quit"}
	case 'S':
		return keyAction{command: "split"}
	case '|':
		return keyAction{command: "split", args: []string{"-v"}}
	case 'X':
		return keyAction{command: "remove"}
	case 'Q':
		return keyAction{command: "only"}
	case '?':
		return keyAction{local: "help"}
	case 'A':
		return keyAction{local: "title"}
	case ':':
		return keyAction{local: "command"}
	case '[':
		return keyAction{local: "copy"}
	case ']':
		return keyAction{command: "paste"}
	case 'h':
		return keyAction{command: "hardcopy"}
	case 'H':
		return keyAction{command: "log"}
	}
	if char >= '0' && char <= '9' {
		return keyAction{command: "select", args: []string{string(char)}}
	}
	if event.Key() == tcell.KeyTAB {
		return keyAction{command: "focus"}
	}
	return keyAction{}
}

func eventRune(event *tcell.EventKey) rune {
	text := event.Str()
	if text == "" {
		return 0
	}
	char, _ := utf8.DecodeRuneInString(text)
	return char
}

func isCtrl(event *tcell.EventKey, char rune) bool {
	if char == 'a' && event.Key() == tcell.KeyCtrlA {
		return true
	}
	return event.Modifiers()&tcell.ModCtrl != 0 && eventRune(event) == char
}

func keyBytes(event *tcell.EventKey) []byte {
	switch event.Key() {
	case tcell.KeyEnter:
		return []byte{'\r'}
	case tcell.KeyTab:
		return []byte{'\t'}
	case tcell.KeyBackspace, tcell.KeyBackspace2:
		return []byte{0x7f}
	case tcell.KeyEscape:
		return []byte{0x1b}
	case tcell.KeyUp:
		return []byte("\x1b[A")
	case tcell.KeyDown:
		return []byte("\x1b[B")
	case tcell.KeyRight:
		return []byte("\x1b[C")
	case tcell.KeyLeft:
		return []byte("\x1b[D")
	case tcell.KeyHome:
		return []byte("\x1b[H")
	case tcell.KeyEnd:
		return []byte("\x1b[F")
	case tcell.KeyPgUp:
		return []byte("\x1b[5~")
	case tcell.KeyPgDn:
		return []byte("\x1b[6~")
	case tcell.KeyDelete:
		return []byte("\x1b[3~")
	}
	if event.Key() >= tcell.KeyCtrlA && event.Key() <= tcell.KeyCtrlZ {
		return []byte{byte(event.Key()-tcell.KeyCtrlA) + 1}
	}
	char := eventRune(event)
	if char == 0 {
		return nil
	}
	buffer := make([]byte, utf8.RuneLen(char))
	utf8.EncodeRune(buffer, char)
	if event.Modifiers()&tcell.ModAlt != 0 {
		return append([]byte{0x1b}, buffer...)
	}
	return buffer
}
