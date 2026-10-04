package client

import (
	"strconv"
	"strings"

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
		return false, s.send(protocol.Request{
			Op: "split-pane", Horizontal: true,
		})
	case value == "\"":
		return false, s.send(protocol.Request{Op: "split-pane"})
	case value == "o":
		return false, s.send(protocol.Request{
			Op: "select-pane", Direction: "next",
		})
	case value == "z":
		return false, s.send(protocol.Request{Op: "zoom-pane"})
	case value == "x":
		return false, s.send(protocol.Request{Op: "kill-pane"})
	case value == "n":
		return false, s.send(protocol.Request{
			Op: "select-window-relative", Direction: "next",
		})
	case value == "p":
		return false, s.send(protocol.Request{
			Op: "select-window-relative", Direction: "previous",
		})
	case value == "l":
		return false, s.send(protocol.Request{Op: "last-window"})
	case value == " ":
		return false, s.send(protocol.Request{
			Op: "select-layout", Name: "next",
		})
	case value == "[":
		id := s.id()
		s.copyRequest = id
		return false, s.send(protocol.Request{
			ID: id, Op: "capture-pane",
		})
	case value == "]":
		return false, s.send(protocol.Request{Op: "paste-buffer"})
	case value == "q":
		s.displayInput = ""
		return false, s.send(protocol.Request{Op: "display-panes"})
	case isCtrl(key, 'b'):
		return false, s.send(protocol.Request{
			Op: "input", Data: "\x02",
		})
	}
	switch key.Key() {
	case tcell.KeyLeft:
		return false, s.send(protocol.Request{
			Op: "select-pane", Direction: "left",
		})
	case tcell.KeyRight:
		return false, s.send(protocol.Request{
			Op: "select-pane", Direction: "right",
		})
	case tcell.KeyUp:
		return false, s.send(protocol.Request{
			Op: "select-pane", Direction: "up",
		})
	case tcell.KeyDown:
		return false, s.send(protocol.Request{
			Op: "select-pane", Direction: "down",
		})
	}
	return false, nil
}

func (s *attachState) displayPanesActive() bool {
	return s.frame != nil && s.frame.DisplayPanes
}

func (s *attachState) handleDisplayKey(key *tcell.EventKey) error {
	if key.Key() == tcell.KeyEsc {
		s.displayInput = ""
		return s.send(protocol.Request{Op: "cancel-display-panes"})
	}
	if key.Key() == tcell.KeyEnter {
		return s.selectDisplayPane()
	}
	if key.Key() != tcell.KeyRune {
		return nil
	}
	value := key.Str()
	if len(value) != 1 || value[0] < '0' || value[0] > '9' {
		return nil
	}
	s.displayInput += value
	exact, candidates := -1, 0
	for _, pane := range s.frame.Panes {
		label := strconv.Itoa(pane.Index)
		if strings.HasPrefix(label, s.displayInput) {
			candidates++
		}
		if label == s.displayInput {
			exact = pane.Index
		}
	}
	if candidates == 0 {
		s.displayInput = ""
		s.draw()
		return nil
	}
	if exact >= 0 && candidates == 1 {
		return s.choosePane(exact)
	}
	s.draw()
	return nil
}

func (s *attachState) selectDisplayPane() error {
	index, err := strconv.Atoi(s.displayInput)
	if err != nil {
		return nil
	}
	for _, pane := range s.frame.Panes {
		if pane.Index == index {
			return s.choosePane(index)
		}
	}
	return nil
}

func (s *attachState) choosePane(index int) error {
	s.displayInput = ""
	if s.frame != nil {
		copy := *s.frame
		copy.DisplayPanes = false
		s.frame = &copy
	}
	return s.send(protocol.Request{Op: "select-pane", Pane: &index})
}

func (s *attachState) handleMouse(event *tcell.EventMouse) error {
	if s.copy != nil {
		return s.handleCopyMouse(event)
	}
	if s.frame == nil {
		return nil
	}
	x, y := event.Position()
	buttons := event.Buttons()
	pane, rect := s.paneAt(x, y)
	if s.displayPanesActive() {
		s.mouseButton = buttons
		if pane >= 0 && buttons&tcell.Button1 != 0 {
			return s.choosePane(pane)
		}
		return nil
	}
	_, rows := s.screen.Size()
	if y == rows-1 && buttons&tcell.Button1 != 0 {
		for _, hit := range s.frame.WindowHits {
			if x >= hit.Start && x < hit.End {
				index := hit.Index
				return s.send(protocol.Request{
					Op: "select-window", Window: &index,
				})
			}
		}
		return nil
	}
	if pane < 0 {
		return nil
	}
	if buttons&tcell.Button1 != 0 &&
		s.mouseButton&tcell.Button1 == 0 {
		if err := s.send(protocol.Request{
			Op: "select-pane", Pane: &pane,
		}); err != nil {
			return err
		}
	}
	mouse, ok := mouseBytes(
		event,
		x-rect.X,
		y-rect.Y,
		s.mouseButton,
	)
	s.mouseButton = buttons
	if !ok {
		return nil
	}
	return s.send(protocol.Request{
		Op: "input", Pane: &pane, Data: string(mouse),
	})
}

func (s *attachState) paneAt(x, y int) (int, protocol.Rect) {
	for _, frame := range s.frame.Panes {
		insideX := x >= frame.Rect.X && x < frame.Rect.X+frame.Rect.W
		insideY := y >= frame.Rect.Y && y < frame.Rect.Y+frame.Rect.H
		if insideX && insideY {
			return frame.Index, frame.Rect
		}
	}
	return -1, protocol.Rect{}
}

func mouseBytes(
	event *tcell.EventMouse,
	x, y int,
	previous tcell.ButtonMask,
) ([]byte, bool) {
	buttons := event.Buttons()
	input := common.MouseEvent{
		Column: uint16(max(x, 0)),
		Row:    uint16(max(y, 0)),
	}
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
