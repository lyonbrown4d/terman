package client

import (
	"fmt"
	"strings"

	"github.com/gdamore/tcell/v3"
	"github.com/lyonbrown4d/terman/screen/internal/proto"
	"github.com/mattn/go-runewidth"
)

type copyPoint struct {
	row int
	col int
}

type copyMode struct {
	lines       []string
	cursor      copyPoint
	start       copyPoint
	selecting   bool
	search      string
	searchInput []rune
	searching   bool
	direction   int
}

func newCopyMode(lines []string) *copyMode {
	if len(lines) == 0 {
		lines = []string{""}
	}
	row := len(lines) - 1
	column := len([]rune(lines[row]))
	point := copyPoint{row: row, col: column}
	return &copyMode{lines: lines, cursor: point, start: point, direction: 1}
}

func (s *attachState) handleCopyKey(key *tcell.EventKey) error {
	copy := s.copy
	if copy.searching {
		return s.handleSearchKey(key)
	}
	switch key.Key() {
	case tcell.KeyEsc:
		s.copy = nil
	case tcell.KeyUp:
		copy.moveRow(-1)
	case tcell.KeyDown:
		copy.moveRow(1)
	case tcell.KeyLeft:
		copy.moveColumn(-1)
	case tcell.KeyRight:
		copy.moveColumn(1)
	case tcell.KeyHome:
		copy.cursor.col = 0
	case tcell.KeyEnd:
		copy.cursor.col = copy.lineLength(copy.cursor.row)
	case tcell.KeyPgUp:
		_, rows := s.screen.Size()
		copy.moveRow(-max(rows-2, 1))
	case tcell.KeyPgDn:
		_, rows := s.screen.Size()
		copy.moveRow(max(rows-2, 1))
	case tcell.KeyEnter:
		if copy.selecting {
			return s.finishCopy()
		}
		copy.start, copy.selecting = copy.cursor, true
	case tcell.KeyRune:
		switch eventRune(key) {
		case 'q':
			s.copy = nil
		case 'h':
			copy.moveColumn(-1)
		case 'j':
			copy.moveRow(1)
		case 'k':
			copy.moveRow(-1)
		case 'l':
			copy.moveColumn(1)
		case 'g':
			copy.cursor = copyPoint{}
		case 'G':
			copy.cursor.row = len(copy.lines) - 1
			copy.cursor.col = copy.lineLength(copy.cursor.row)
		case ' ':
			if copy.selecting {
				return s.finishCopy()
			}
			copy.start, copy.selecting = copy.cursor, true
		case '/', '?':
			copy.searching = true
			copy.searchInput = []rune{}
			if eventRune(key) == '?' {
				copy.direction = -1
			} else {
				copy.direction = 1
			}
		case 'n':
			copy.find(copy.direction)
		case 'N':
			copy.find(-copy.direction)
		}
	}
	s.draw()
	return nil
}

func (s *attachState) finishCopy() error {
	data := []byte(s.copy.selection())
	s.screen.SetClipboard(data)
	s.copy = nil
	return s.send(proto.Request{Type: "command", Command: "copy", Data: data})
}

func (s *attachState) handleSearchKey(key *tcell.EventKey) error {
	copy := s.copy
	switch key.Key() {
	case tcell.KeyEsc:
		copy.searching = false
	case tcell.KeyEnter:
		copy.searching = false
		copy.search = string(copy.searchInput)
		copy.find(copy.direction)
	case tcell.KeyBackspace, tcell.KeyBackspace2:
		if len(copy.searchInput) > 0 {
			copy.searchInput = copy.searchInput[:len(copy.searchInput)-1]
		}
	case tcell.KeyRune:
		copy.searchInput = append(copy.searchInput, eventRune(key))
	}
	s.draw()
	return nil
}

func (c *copyMode) moveRow(delta int) {
	c.cursor.row = min(max(c.cursor.row+delta, 0), len(c.lines)-1)
	c.cursor.col = min(c.cursor.col, c.lineLength(c.cursor.row))
}

func (c *copyMode) moveColumn(delta int) {
	c.cursor.col += delta
	if c.cursor.col < 0 && c.cursor.row > 0 {
		c.cursor.row--
		c.cursor.col = c.lineLength(c.cursor.row)
	}
	if c.cursor.col > c.lineLength(c.cursor.row) && c.cursor.row < len(c.lines)-1 {
		c.cursor.row++
		c.cursor.col = 0
	}
	c.cursor.col = min(max(c.cursor.col, 0), c.lineLength(c.cursor.row))
}

func (c *copyMode) lineLength(row int) int {
	return len([]rune(c.lines[row]))
}

func (c *copyMode) find(direction int) {
	if c.search == "" {
		return
	}
	for step := 1; step <= len(c.lines); step++ {
		row := (c.cursor.row + direction*step + len(c.lines)*2) % len(c.lines)
		column := strings.Index(c.lines[row], c.search)
		if column >= 0 {
			c.cursor.row = row
			c.cursor.col = len([]rune(c.lines[row][:column]))
			return
		}
	}
}

func (c *copyMode) selection() string {
	first, last := c.start, c.cursor
	if pointAfter(first, last) {
		first, last = last, first
	}
	selected := make([]string, 0, last.row-first.row+1)
	for row := first.row; row <= last.row; row++ {
		runes := []rune(c.lines[row])
		start, end := 0, len(runes)
		if row == first.row {
			start = min(first.col, len(runes))
		}
		if row == last.row {
			end = min(last.col+1, len(runes))
		}
		if end < start {
			end = start
		}
		selected = append(selected, string(runes[start:end]))
	}
	return strings.Join(selected, "\n")
}

func pointAfter(first, second copyPoint) bool {
	return first.row > second.row || first.row == second.row && first.col > second.col
}

func (s *attachState) drawCopy() {
	cols, rows := s.screen.Size()
	bodyRows := max(rows-1, 1)
	top := max(s.copy.cursor.row-bodyRows+1, 0)
	if top+bodyRows > len(s.copy.lines) {
		top = max(len(s.copy.lines)-bodyRows, 0)
	}
	left := max(s.copy.cursor.col-cols+1, 0)
	for y := 0; y < bodyRows; y++ {
		row := top + y
		if row >= len(s.copy.lines) {
			continue
		}
		s.drawCopyLine(row, y, left, cols)
	}
	status := fmt.Sprintf(
		"copy mode | find %s | line %d/%d | /? search n/N repeat | Space mark/copy Enter copy Esc cancel",
		s.copy.search,
		s.copy.cursor.row+1,
		len(s.copy.lines),
	)
	if s.copy.searching {
		prefix := "/"
		if s.copy.direction < 0 {
			prefix = "?"
		}
		status = prefix + string(s.copy.searchInput)
	}
	drawText(s.screen, 0, rows-1, cols, status, tcell.StyleDefault.Reverse(true))
}

func (s *attachState) drawCopyLine(row, y, left, width int) {
	runes := []rune(s.copy.lines[row])
	column := 0
	for index := left; index < len(runes) && column < width; index++ {
		char := runes[index]
		charWidth := max(runewidth.RuneWidth(char), 1)
		if column+charWidth > width {
			break
		}
		style := tcell.StyleDefault
		point := copyPoint{row: row, col: index}
		if s.copy.selecting && s.copy.selected(point) {
			style = style.Reverse(true)
		}
		if point == s.copy.cursor {
			style = style.Underline(true)
		}
		s.screen.SetContent(column, y, char, nil, style)
		column += charWidth
	}
}

func (c *copyMode) selected(point copyPoint) bool {
	first, last := c.start, c.cursor
	if pointAfter(first, last) {
		first, last = last, first
	}
	return !pointAfter(first, point) && !pointAfter(point, last)
}
