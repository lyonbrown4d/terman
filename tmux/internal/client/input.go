package client

import (
	"fmt"

	"github.com/gdamore/tcell/v3"
	"github.com/lyonbrown4d/terman/common"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
)

func (s *attachState) handlePrefix(key *tcell.EventKey) (bool, error) {
	value := key.Str()
	switch {
	case value == "d":
		return true, s.send(protocol.Request{Op: "detach"})
	case value == "c":
		return false, s.send(protocol.Request{Op: "new-window"})
	case value == "%":
		return false, s.send(protocol.Request{Op: "split-pane", Horizontal: true})
	case value == "\"":
		return false, s.send(protocol.Request{Op: "split-pane"})
	case value == "o":
		return false, s.send(protocol.Request{Op: "select-pane", Direction: "next"})
	case value == "z":
		return false, s.send(protocol.Request{Op: "zoom-pane"})
	case value == "x":
		return false, s.send(protocol.Request{Op: "kill-pane"})
	case value == "n":
		return false, s.send(protocol.Request{Op: "select-window-relative", Direction: "next"})
	case value == "p":
		return false, s.send(protocol.Request{Op: "select-window-relative", Direction: "previous"})
	case value == "l":
		return false, s.send(protocol.Request{Op: "last-window"})
	case value == " ":
		return false, s.send(protocol.Request{Op: "select-layout", Name: "next"})
	case value == "[":
		id := s.id()
		s.copyRequest = id
		return false, s.send(protocol.Request{ID: id, Op: "capture-pane"})
	case value == "]":
		return false, s.send(protocol.Request{Op: "paste-buffer"})
	case isCtrl(key, 'b'):
		return false, s.send(protocol.Request{Op: "input", Data: "\x02"})
	}
	switch key.Key() {
	case tcell.KeyLeft:
		return false, s.send(protocol.Request{Op: "select-pane", Direction: "left"})
	case tcell.KeyRight:
		return false, s.send(protocol.Request{Op: "select-pane", Direction: "right"})
	case tcell.KeyUp:
		return false, s.send(protocol.Request{Op: "select-pane", Direction: "up"})
	case tcell.KeyDown:
		return false, s.send(protocol.Request{Op: "select-pane", Direction: "down"})
	}
	return false, nil
}

func (s *attachState) handleMouse(event *tcell.EventMouse) error {
	if s.frame == nil {
		return nil
	}
	x, y := event.Position()
	buttons := event.Buttons()
	_, rows := s.screen.Size()
	if y == rows-1 && buttons&tcell.Button1 != 0 {
		for _, hit := range s.frame.WindowHits {
			if x >= hit.Start && x < hit.End {
				index := hit.Index
				return s.send(protocol.Request{Op: "select-window", Window: &index})
			}
		}
		return nil
	}
	pane := -1
	var rect protocol.Rect
	for _, frame := range s.frame.Panes {
		if x >= frame.Rect.X && x < frame.Rect.X+frame.Rect.W && y >= frame.Rect.Y && y < frame.Rect.Y+frame.Rect.H {
			pane, rect = frame.Index, frame.Rect
			break
		}
	}
	if pane < 0 {
		return nil
	}
	if buttons&tcell.Button1 != 0 && s.mouseButton&tcell.Button1 == 0 {
		if err := s.send(protocol.Request{Op: "select-pane", Pane: &pane}); err != nil {
			return err
		}
	}
	mouse, ok := mouseBytes(event, x-rect.X, y-rect.Y, s.mouseButton)
	s.mouseButton = buttons
	if !ok {
		return nil
	}
	return s.send(protocol.Request{Op: "input", Pane: &pane, Data: string(mouse)})
}

func mouseBytes(event *tcell.EventMouse, x, y int, previous tcell.ButtonMask) ([]byte, bool) {
	buttons := event.Buttons()
	input := common.MouseEvent{Column: uint16(max(x, 0)), Row: uint16(max(y, 0))}
	switch {
	case buttons&tcell.WheelUp != 0:
		input.Kind = common.MouseScrollUp
	case buttons&tcell.WheelDown != 0:
		input.Kind = common.MouseScrollDown
	case buttons&tcell.Button1 != 0:
		input.Button = common.MouseButtonLeft
		if previous&tcell.Button1 != 0 {
			input.Kind = common.MouseDrag
		} else {
			input.Kind = common.MouseDown
		}
	case previous&tcell.Button1 != 0:
		input.Button, input.Kind = common.MouseButtonLeft, common.MouseUp
	case buttons&tcell.Button2 != 0:
		input.Button, input.Kind = common.MouseButtonRight, common.MouseDown
	case buttons&tcell.Button3 != 0:
		input.Button, input.Kind = common.MouseButtonMiddle, common.MouseDown
	default:
		return nil, false
	}
	if event.Modifiers()&tcell.ModCtrl != 0 {
		input.Modifiers |= common.ModifierControl
	}
	if event.Modifiers()&tcell.ModAlt != 0 {
		input.Modifiers |= common.ModifierAlt
	}
	if event.Modifiers()&tcell.ModShift != 0 {
		input.Modifiers |= common.ModifierShift
	}
	return common.MouseEventBytes(input)
}

func (s *attachState) draw() {
	s.screen.Clear()
	if s.copy != nil {
		s.drawCopy()
		s.screen.Show()
		return
	}
	if s.frame == nil {
		s.screen.Show()
		return
	}
	for _, pane := range s.frame.Panes {
		style := tcell.StyleDefault
		for row, line := range pane.Lines {
			drawText(s.screen, pane.Rect.X, pane.Rect.Y+row, pane.Rect.W, line, style)
		}
		border := tcell.StyleDefault.Foreground(tcell.ColorGray)
		if pane.Active {
			border = border.Foreground(tcell.ColorGreen).Bold(true)
		}
		label := fmt.Sprintf("[%d]", pane.Index)
		drawText(s.screen, pane.Rect.X, pane.Rect.Y, min(len(label), pane.Rect.W), label, border)
	}
	cols, rows := s.screen.Size()
	status := s.frame.Status
	if s.frame.Message != "" {
		status = s.frame.Message
	}
	drawText(s.screen, 0, rows-1, cols, common.FitTerminalText(status, cols), tcell.StyleDefault.Reverse(true))
	s.screen.Show()
}

func drawText(screen tcell.Screen, x, y, width int, value string, style tcell.Style) {
	column := 0
	for _, char := range value {
		if column >= width {
			break
		}
		screen.SetContent(x+column, y, char, nil, style)
		column++
	}
	for column < width {
		screen.SetContent(x+column, y, ' ', nil, style)
		column++
	}
}
