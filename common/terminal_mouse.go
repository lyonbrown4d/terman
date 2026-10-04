package common

import (
	"fmt"
	"io"
	"os"
)

// MouseButton identifies a terminal mouse button.
type MouseButton uint8

const (
	MouseButtonLeft MouseButton = iota
	MouseButtonMiddle
	MouseButtonRight
)

// MouseEventKind identifies an SGR mouse action.
type MouseEventKind uint8

const (
	MouseDown MouseEventKind = iota
	MouseUp
	MouseDrag
	MouseMoved
	MouseScrollUp
	MouseScrollDown
	MouseScrollLeft
	MouseScrollRight
)

// KeyModifiers is a bit set of keyboard modifiers attached to a mouse event.
type KeyModifiers uint8

const (
	ModifierShift KeyModifiers = 1 << iota
	ModifierAlt
	ModifierControl
)

// MouseEvent is a zero-based terminal mouse event.
type MouseEvent struct {
	Kind      MouseEventKind
	Button    MouseButton
	Column    uint16
	Row       uint16
	Modifiers KeyModifiers
}

// EnableMouseCapture enables button, drag, and SGR mouse reporting on stdout.
func EnableMouseCapture() error {
	return EnableMouseCaptureTo(os.Stdout)
}

// EnableMouseCaptureTo enables mouse reporting on writer.
func EnableMouseCaptureTo(writer io.Writer) error {
	_, err := io.WriteString(writer, "\x1b[?1000h\x1b[?1002h\x1b[?1006h")
	return err
}

// DisableMouseCapture disables SGR, drag, and button mouse reporting on stdout.
func DisableMouseCapture() {
	_ = DisableMouseCaptureTo(os.Stdout)
}

// DisableMouseCaptureTo disables mouse reporting on writer.
func DisableMouseCaptureTo(writer io.Writer) error {
	_, err := io.WriteString(writer, "\x1b[?1006l\x1b[?1002l\x1b[?1000l")
	return err
}

// MouseEventBytes encodes an event using xterm SGR mouse protocol.
// Buttonless move events and invalid buttons are not encoded.
func MouseEventBytes(event MouseEvent) ([]byte, bool) {
	modifier := modifierCode(event.Modifiers)
	code := uint16(0)
	suffix := byte('M')

	switch event.Kind {
	case MouseDown, MouseUp, MouseDrag:
		button, ok := buttonCode(event.Button)
		if !ok {
			return nil, false
		}
		code = button + modifier
		if event.Kind == MouseUp {
			suffix = 'm'
		}
		if event.Kind == MouseDrag {
			code += 32
		}
	case MouseMoved:
		return nil, false
	case MouseScrollUp:
		code = modifier + 64
	case MouseScrollDown:
		code = modifier + 65
	case MouseScrollLeft:
		code = modifier + 66
	case MouseScrollRight:
		code = modifier + 67
	default:
		return nil, false
	}

	column := uint32(event.Column) + 1
	row := uint32(event.Row) + 1
	return fmt.Appendf(nil, "\x1b[<%d;%d;%d%c", code, column, row, suffix), true
}

func buttonCode(button MouseButton) (uint16, bool) {
	switch button {
	case MouseButtonLeft:
		return 0, true
	case MouseButtonMiddle:
		return 1, true
	case MouseButtonRight:
		return 2, true
	default:
		return 0, false
	}
}

func modifierCode(modifiers KeyModifiers) uint16 {
	code := uint16(0)
	if modifiers&ModifierShift != 0 {
		code += 4
	}
	if modifiers&ModifierAlt != 0 {
		code += 8
	}
	if modifiers&ModifierControl != 0 {
		code += 16
	}
	return code
}
