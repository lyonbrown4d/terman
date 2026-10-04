package client

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/gdamore/tcell/v3"
	"github.com/lyonbrown4d/terman/screen/internal/proto"
	"github.com/mattn/go-runewidth"
)

const (
	attrBold = 1 << iota
	attrFaint
	attrItalic
	attrBlink
	attrRapidBlink
	attrReverse
	attrConceal
	attrStrike
)

func (s *attachState) draw() {
	s.screen.Clear()
	s.screen.HideCursor()
	switch {
	case s.copy != nil:
		s.drawCopy()
	case s.help:
		s.drawHelp()
	default:
		s.drawFrame()
		if s.prompt != nil {
			s.drawPrompt()
		}
	}
	s.screen.Show()
}

func (s *attachState) drawFrame() {
	if s.frame == nil {
		return
	}
	for _, region := range s.frame.Regions {
		for y := 0; y < region.Height; y++ {
			if y < len(region.Cells) && len(region.Cells[y]) > 0 {
				drawCells(s.screen, region.X, region.Y+y, region.Width, region.Cells[y])
			} else if y < len(region.Lines) {
				drawText(
					s.screen,
					region.X,
					region.Y+y,
					region.Width,
					region.Lines[y],
					tcell.StyleDefault,
				)
			}
		}
		if len(s.frame.Regions) > 1 {
			style := tcell.StyleDefault.Foreground(tcell.ColorGray)
			if region.Focused {
				style = style.Foreground(tcell.ColorGreen).Bold(true)
			}
			label := fmt.Sprintf("[%d]", region.Index)
			drawText(s.screen, region.X, region.Y, min(len(label), region.Width), label, style)
		}
	}
	cols, rows := s.screen.Size()
	status := fmt.Sprintf("screen %s | %s | Ctrl-A ? help", s.frame.Session, windowStatus(s.frame))
	if s.frame.Status != "" {
		status = s.frame.Status
	}
	if s.notice != "" {
		status = s.notice
	}
	drawText(s.screen, 0, rows-1, cols, status, tcell.StyleDefault.Reverse(true))
}

func drawCells(screen tcell.Screen, x, y, width int, cells []proto.Cell) {
	for _, cell := range cells {
		if cell.X < 0 || cell.X >= width {
			continue
		}
		runes := []rune(cell.Text)
		if len(runes) == 0 || cell.Attributes&attrConceal != 0 {
			runes = []rune{' '}
		}
		screen.SetContent(x+cell.X, y, runes[0], runes[1:], cellStyle(cell))
	}
}

func cellStyle(cell proto.Cell) tcell.Style {
	style := tcell.StyleDefault
	if cell.Foreground != "" {
		style = style.Foreground(tcell.GetColor(cell.Foreground))
	}
	if cell.Background != "" {
		style = style.Background(tcell.GetColor(cell.Background))
	}
	style = style.Bold(cell.Attributes&attrBold != 0)
	style = style.Dim(cell.Attributes&attrFaint != 0)
	style = style.Italic(cell.Attributes&attrItalic != 0)
	style = style.Blink(cell.Attributes&(attrBlink|attrRapidBlink) != 0)
	style = style.Reverse(cell.Attributes&attrReverse != 0)
	style = style.StrikeThrough(cell.Attributes&attrStrike != 0)
	if cell.Underline != 0 {
		if cell.UnderlineColor != "" {
			style = style.Underline(tcell.GetColor(cell.UnderlineColor))
		} else {
			style = style.Underline(true)
		}
	}
	return style
}

func drawText(screen tcell.Screen, x, y, width int, text string, style tcell.Style) {
	column := 0
	for _, char := range text {
		charWidth := max(runewidth.RuneWidth(char), 1)
		if column+charWidth > width {
			break
		}
		screen.SetContent(x+column, y, char, nil, style)
		column += charWidth
	}
	for column < width {
		screen.SetContent(x+column, y, ' ', nil, style)
		column++
	}
}

func windowStatus(frame *proto.Frame) string {
	parts := make([]string, 0, len(frame.Windows))
	for _, window := range frame.Windows {
		marker := "-"
		if window.Active {
			marker = "*"
		}
		parts = append(parts, strconv.Itoa(window.Index)+marker+window.Title)
	}
	return strings.Join(parts, " ")
}

func (s *attachState) drawHelp() {
	cols, rows := s.screen.Size()
	lines := []string{
		"terman-screen help (press any key to close)",
		"Ctrl-A c new | d detach | n/p next/prev | 0..9 select | k kill",
		"Ctrl-A S horizontal split | | vertical split | Tab focus | X remove | Q only",
		"Ctrl-A [ or Esc copy mode | ] paste buffer | h hardcopy | H toggle log",
		"Ctrl-A A title prompt | : command prompt | ? this help",
		"copy: arrows/hjkl move, Home/End, PgUp/PgDn, Space mark, Enter copy",
		"copy search: / forward, ? backward, n repeat, N reverse repeat",
	}
	for index, line := range lines {
		if index >= rows-1 {
			break
		}
		style := tcell.StyleDefault
		if index == 0 {
			style = style.Bold(true).Reverse(true)
		}
		drawText(s.screen, 0, index, cols, line, style)
	}
	drawText(s.screen, 0, rows-1, cols, "Ctrl-A ? help", tcell.StyleDefault.Reverse(true))
}
