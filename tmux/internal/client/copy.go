package client

import (
	"strings"

	"github.com/gdamore/tcell/v3"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
)

type copyMode struct {
	lines           [][]rune
	row             int
	column          int
	anchorRow       int
	anchorColumn    int
	selecting       bool
	searching       bool
	searchDirection int
	query           []rune
	lastQuery       string
	lastDirection   int
	top             int
}

func newCopyMode(data string) *copyMode {
	data = strings.ReplaceAll(data, "\r\n", "\n")
	values := strings.Split(data, "\n")
	if len(values) == 0 {
		values = []string{""}
	}
	lines := make([][]rune, len(values))
	for i, value := range values {
		lines[i] = []rune(value)
	}
	row := len(lines) - 1
	column := max(len(lines[row])-1, 0)
	return &copyMode{
		lines: lines, row: row, column: column,
		anchorRow: row, anchorColumn: column,
	}
}

func (s *attachState) handleCopyKey(key *tcell.EventKey) error {
	if s.copy.searching {
		return s.handleSearchKey(key)
	}
	switch key.Key() {
	case tcell.KeyEsc:
		s.copy = nil
	case tcell.KeyUp:
		s.copy.moveVertical(-1)
	case tcell.KeyDown:
		s.copy.moveVertical(1)
	case tcell.KeyLeft:
		s.copy.moveHorizontal(-1)
	case tcell.KeyRight:
		s.copy.moveHorizontal(1)
	case tcell.KeyPgUp:
		_, rows := s.screen.Size()
		s.copy.moveVertical(-max(rows-1, 1))
	case tcell.KeyPgDn:
		_, rows := s.screen.Size()
		s.copy.moveVertical(max(rows-1, 1))
	case tcell.KeyEnter:
		return s.finishCopy()
	case tcell.KeyRune:
		s.handleCopyRune(key.Str())
	}
	s.draw()
	return nil
}

func (s *attachState) handleCopyRune(value string) {
	switch value {
	case "q":
		s.copy = nil
	case "k":
		s.copy.moveVertical(-1)
	case "j":
		s.copy.moveVertical(1)
	case "h":
		s.copy.moveHorizontal(-1)
	case "l":
		s.copy.moveHorizontal(1)
	case "g":
		s.copy.row, s.copy.column = 0, 0
	case "G":
		s.copy.row = len(s.copy.lines) - 1
		s.copy.column = max(len(s.copy.lines[s.copy.row])-1, 0)
	case "0":
		s.copy.column = 0
	case "$":
		s.copy.column = max(len(s.copy.lines[s.copy.row])-1, 0)
	case "w":
		s.copy.moveWord(1, false)
	case "e":
		s.copy.moveWord(1, true)
	case "b":
		s.copy.moveWord(-1, false)
	case " ":
		s.copy.anchorRow = s.copy.row
		s.copy.anchorColumn = s.copy.column
		s.copy.selecting = !s.copy.selecting
	case "/", "?":
		s.copy.searching = true
		s.copy.query = []rune{}
		s.copy.searchDirection = 1
		if value == "?" {
			s.copy.searchDirection = -1
		}
	case "n":
		s.copy.repeatSearch(false)
	case "N":
		s.copy.repeatSearch(true)
	}
}

func (s *attachState) handleSearchKey(key *tcell.EventKey) error {
	switch key.Key() {
	case tcell.KeyEsc:
		s.copy.searching = false
		s.copy.query = []rune{}
	case tcell.KeyEnter:
		query := string(s.copy.query)
		s.copy.searching = false
		if query != "" {
			s.copy.lastQuery = query
			s.copy.lastDirection = s.copy.searchDirection
			s.copy.search(query, s.copy.searchDirection)
		}
	case tcell.KeyBackspace, tcell.KeyBackspace2:
		if len(s.copy.query) > 0 {
			s.copy.query = s.copy.query[:len(s.copy.query)-1]
		}
	case tcell.KeyRune:
		s.copy.query = append(s.copy.query, []rune(key.Str())...)
	}
	s.draw()
	return nil
}

func (s *attachState) finishCopy() error {
	data := s.copy.selectionText()
	s.copy = nil
	s.draw()
	return s.send(protocol.Request{Op: "set-buffer", Data: data})
}
