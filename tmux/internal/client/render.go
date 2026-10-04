package client

import (
	"fmt"

	"github.com/gdamore/tcell/v3"
	"github.com/lyonbrown4d/terman/common"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
	"github.com/mattn/go-runewidth"
)

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
		drawCells(s.screen, pane)
	}
	if s.frame.DisplayPanes {
		s.drawPaneOverlays()
	}
	cols, rows := s.screen.Size()
	status := s.frame.Status
	if s.frame.Message != "" {
		status = s.frame.Message
	}
	if s.frame.DisplayPanes {
		status = "[display-panes] number + Enter or click"
		if s.displayInput != "" {
			status += ": " + s.displayInput
		}
	}
	drawText(
		s.screen,
		0,
		rows-1,
		cols,
		common.FitTerminalText(status, cols),
		tcell.StyleDefault.Reverse(true),
	)
	s.screen.Show()
}

func drawCells(screen tcell.Screen, pane protocol.PaneFrame) {
	for y, row := range pane.Cells {
		if y >= pane.Rect.H {
			break
		}
		for x, cell := range row {
			if x >= pane.Rect.W || cell.Width == 0 || cell.Content == "" {
				continue
			}
			runes := []rune(cell.Content)
			if len(runes) == 0 {
				continue
			}
			screen.SetContent(
				pane.Rect.X+x,
				pane.Rect.Y+y,
				runes[0],
				runes[1:],
				tcellStyle(cell.Style),
			)
		}
	}
}

func tcellStyle(value protocol.CellStyle) tcell.Style {
	style := tcell.StyleDefault
	if value.HasForeground {
		style = style.Foreground(rgbColor(value.Foreground))
	}
	if value.HasBackground {
		style = style.Background(rgbColor(value.Background))
	}
	return style.
		Bold(value.Bold).
		Underline(value.Underline).
		Reverse(value.Reverse)
}

func rgbColor(value uint32) tcell.Color {
	return tcell.NewRGBColor(
		int32(value>>16&0xff),
		int32(value>>8&0xff),
		int32(value&0xff),
	)
}

func (s *attachState) drawPaneOverlays() {
	for _, pane := range s.frame.Panes {
		label := fmt.Sprintf(" %d ", pane.Index)
		width := runewidth.StringWidth(label)
		x := pane.Rect.X + max((pane.Rect.W-width)/2, 0)
		y := pane.Rect.Y + max(pane.Rect.H/2, 0)
		style := tcell.StyleDefault.
			Foreground(tcell.ColorWhite).
			Background(tcell.ColorBlue).
			Bold(true)
		drawText(s.screen, x, y, min(width, pane.Rect.W), label, style)
	}
}

func drawText(
	screen tcell.Screen,
	x, y, width int,
	value string,
	style tcell.Style,
) {
	column := 0
	for _, char := range value {
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
