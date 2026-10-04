package client

import (
	"strings"
	"unicode"
)

func (c *copyMode) moveVertical(delta int) {
	c.row = min(max(c.row+delta, 0), len(c.lines)-1)
	c.column = min(c.column, max(len(c.lines[c.row])-1, 0))
}

func (c *copyMode) moveHorizontal(delta int) {
	if delta < 0 && c.column == 0 && c.row > 0 {
		c.row--
		c.column = max(len(c.lines[c.row])-1, 0)
		return
	}
	atEnd := c.column >= max(len(c.lines[c.row])-1, 0)
	if delta > 0 && atEnd && c.row < len(c.lines)-1 {
		c.row++
		c.column = 0
		return
	}
	c.column = min(
		max(c.column+delta, 0),
		max(len(c.lines[c.row])-1, 0),
	)
}

func (c *copyMode) moveWord(direction int, end bool) {
	for c.step(direction) {
		if end {
			if isWordAt(c) && !isWordNext(c, direction) {
				return
			}
			continue
		}
		if isWordAt(c) && !isWordNext(c, -direction) {
			return
		}
	}
}

func (c *copyMode) step(direction int) bool {
	beforeRow, beforeColumn := c.row, c.column
	c.moveHorizontal(direction)
	return beforeRow != c.row || beforeColumn != c.column
}

func isWordAt(c *copyMode) bool {
	line := c.lines[c.row]
	return c.column < len(line) && isWordRune(line[c.column])
}

func isWordNext(c *copyMode, direction int) bool {
	row, column := c.row, c.column
	if !c.step(direction) {
		return false
	}
	word := isWordAt(c)
	c.row, c.column = row, column
	return word
}

func isWordRune(value rune) bool {
	return value == '_' || unicode.IsLetter(value) || unicode.IsNumber(value)
}

func (c *copyMode) repeatSearch(reverse bool) {
	if c.lastQuery == "" {
		return
	}
	direction := c.lastDirection
	if reverse {
		direction = -direction
	}
	c.search(c.lastQuery, direction)
}

func (c *copyMode) search(query string, direction int) bool {
	for offset := 1; offset <= len(c.lines); offset++ {
		row := (c.row + direction*offset + len(c.lines)) % len(c.lines)
		line := string(c.lines[row])
		index := -1
		if direction > 0 {
			index = strings.Index(line, query)
		} else {
			index = strings.LastIndex(line, query)
		}
		if index >= 0 {
			c.row = row
			c.column = len([]rune(line[:index]))
			return true
		}
	}
	return false
}

func (c *copyMode) selectionText() string {
	if !c.selecting {
		return string(c.lines[c.row])
	}
	firstRow, firstColumn := c.anchorRow, c.anchorColumn
	lastRow, lastColumn := c.row, c.column
	if firstRow > lastRow ||
		(firstRow == lastRow && firstColumn > lastColumn) {
		firstRow, lastRow = lastRow, firstRow
		firstColumn, lastColumn = lastColumn, firstColumn
	}
	var output strings.Builder
	for row := firstRow; row <= lastRow; row++ {
		start, end := 0, len(c.lines[row])
		if row == firstRow {
			start = min(firstColumn, end)
		}
		if row == lastRow {
			end = min(lastColumn+1, end)
		}
		if start < end {
			output.WriteString(string(c.lines[row][start:end]))
		}
		if row < lastRow {
			output.WriteByte('\n')
		}
	}
	return output.String()
}
