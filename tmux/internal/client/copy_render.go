package client

import (
	"github.com/gdamore/tcell/v3"
	"github.com/lyonbrown4d/terman/common"
	"github.com/mattn/go-runewidth"
)

func (s *attachState) handleCopyMouse(event *tcell.EventMouse) error {
	buttons := event.Buttons()
	x, y := event.Position()
	if buttons&tcell.WheelUp != 0 {
		s.copy.moveVertical(-3)
		s.draw()
		return nil
	}
	if buttons&tcell.WheelDown != 0 {
		s.copy.moveVertical(3)
		s.draw()
		return nil
	}
	row, column := s.copy.positionAt(x, y)
	if buttons&tcell.Button1 != 0 {
		if s.mouseButton&tcell.Button1 == 0 {
			s.copy.anchorRow, s.copy.anchorColumn = row, column
			s.copy.selecting = true
		}
		s.copy.row, s.copy.column = row, column
		s.mouseButton = buttons
		s.draw()
		return nil
	}
	if s.mouseButton&tcell.Button1 != 0 {
		s.copy.row, s.copy.column = row, column
		s.mouseButton = buttons
		return s.finishCopy()
	}
	s.mouseButton = buttons
	return nil
}

func (c *copyMode) positionAt(x, y int) (int, int) {
	row := min(max(c.top+y, 0), len(c.lines)-1)
	column, visual := 0, 0
	for index, value := range c.lines[row] {
		width := max(runewidth.RuneWidth(value), 1)
		if x < visual+width {
			return row, index
		}
		visual += width
		column = index
	}
	return row, column
}

func (s *attachState) drawCopy() {
	cols, rows := s.screen.Size()
	bodyRows := max(rows-1, 1)
	s.copy.top = max(s.copy.row-bodyRows+1, 0)
	if s.copy.top+bodyRows > len(s.copy.lines) {
		s.copy.top = max(len(s.copy.lines)-bodyRows, 0)
	}
	for y := range bodyRows {
		drawText(s.screen, 0, y, cols, "", tcell.StyleDefault)
		line := s.copy.top + y
		if line < len(s.copy.lines) {
			s.drawCopyLine(line, y, cols)
		}
	}
	status := "[copy-mode] hjkl/wbe move, Space select, /? search, nN repeat, Enter copy"
	if s.copy.searching {
		prefix := "/"
		if s.copy.searchDirection < 0 {
			prefix = "?"
		}
		status = prefix + string(s.copy.query)
	}
	drawText(
		s.screen,
		0,
		rows-1,
		cols,
		common.FitTerminalText(status, cols),
		tcell.StyleDefault.Reverse(true),
	)
}

func (s *attachState) drawCopyLine(line, y, width int) {
	column := 0
	for index, value := range s.copy.lines[line] {
		charWidth := max(runewidth.RuneWidth(value), 1)
		if column+charWidth > width {
			break
		}
		style := tcell.StyleDefault
		if s.copy.selected(line, index) {
			style = style.Reverse(true)
		}
		if line == s.copy.row && index == s.copy.column {
			style = style.Underline(true)
		}
		s.screen.SetContent(column, y, value, nil, style)
		column += charWidth
	}
}

func (c *copyMode) selected(row, column int) bool {
	if !c.selecting {
		return false
	}
	firstRow, firstColumn := c.anchorRow, c.anchorColumn
	lastRow, lastColumn := c.row, c.column
	if firstRow > lastRow ||
		(firstRow == lastRow && firstColumn > lastColumn) {
		firstRow, lastRow = lastRow, firstRow
		firstColumn, lastColumn = lastColumn, firstColumn
	}
	if row < firstRow || row > lastRow {
		return false
	}
	if row == firstRow && column < firstColumn {
		return false
	}
	return row != lastRow || column <= lastColumn
}
