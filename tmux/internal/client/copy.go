package client

import (
	"strings"

	"github.com/gdamore/tcell/v3"
	"github.com/lyonbrown4d/terman/common"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
)

type copyMode struct {
	lines     []string
	row       int
	start     int
	selecting bool
}

func newCopyMode(data string) *copyMode {
	data = strings.ReplaceAll(data, "\r\n", "\n")
	lines := strings.Split(data, "\n")
	row := max(len(lines)-1, 0)
	return &copyMode{lines: lines, row: row, start: row}
}

func (s *attachState) handleCopyKey(key *tcell.EventKey) error {
	switch key.Key() {
	case tcell.KeyEsc:
		s.copy = nil
	case tcell.KeyUp:
		s.copy.row = max(s.copy.row-1, 0)
	case tcell.KeyDown:
		s.copy.row = min(s.copy.row+1, len(s.copy.lines)-1)
	case tcell.KeyPgUp:
		_, rows := s.screen.Size()
		s.copy.row = max(s.copy.row-rows+1, 0)
	case tcell.KeyPgDn:
		_, rows := s.screen.Size()
		s.copy.row = min(s.copy.row+rows-1, len(s.copy.lines)-1)
	case tcell.KeyRune:
		switch key.Str() {
		case "q":
			s.copy = nil
		case "k":
			s.copy.row = max(s.copy.row-1, 0)
		case "j":
			s.copy.row = min(s.copy.row+1, len(s.copy.lines)-1)
		case "g":
			s.copy.row = 0
		case "G":
			s.copy.row = len(s.copy.lines) - 1
		case " ":
			s.copy.start, s.copy.selecting = s.copy.row, !s.copy.selecting
		}
	case tcell.KeyEnter:
		first, last := s.copy.start, s.copy.row
		if first > last {
			first, last = last, first
		}
		data := strings.Join(s.copy.lines[first:last+1], "\n")
		s.copy = nil
		return s.send(protocol.Request{Op: "set-buffer", Data: data})
	}
	s.draw()
	return nil
}

func (s *attachState) drawCopy() {
	cols, rows := s.screen.Size()
	bodyRows := max(rows-1, 1)
	top := max(s.copy.row-bodyRows+1, 0)
	if top+bodyRows > len(s.copy.lines) {
		top = max(len(s.copy.lines)-bodyRows, 0)
	}
	first, last := s.copy.start, s.copy.row
	if first > last {
		first, last = last, first
	}
	for y := 0; y < bodyRows; y++ {
		line := top + y
		style := tcell.StyleDefault
		if s.copy.selecting && line >= first && line <= last {
			style = style.Reverse(true)
		}
		if line == s.copy.row {
			style = style.Underline(true)
		}
		value := ""
		if line < len(s.copy.lines) {
			value = s.copy.lines[line]
		}
		drawText(s.screen, 0, y, cols, common.FitTerminalText(value, cols), style)
	}
	status := common.FitTerminalText("[copy-mode] arrows/jk move, Space select, Enter copy, q cancel", cols)
	drawText(s.screen, 0, rows-1, cols, status, tcell.StyleDefault.Reverse(true))
}
