package session

import (
	"fmt"
	"image/color"
	"strings"
	"time"

	uv "github.com/charmbracelet/ultraviolet"
	"github.com/lyonbrown4d/terman/common"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
)

func (r *Reactor) frame() protocol.Frame {
	now := time.Now()
	window := r.state.Windows[r.state.ActiveWindow]
	frame := protocol.Frame{
		Session:      r.state.Name,
		Window:       r.state.ActiveWindow,
		Status:       r.statusText(),
		Message:      r.state.Message,
		DisplayPanes: now.Before(r.state.DisplayPanesUntil),
		GeneratedAt:  now,
	}
	if window == nil {
		return frame
	}
	frame.ActivePane = window.ActivePane
	rects := r.layoutRects(window)
	frame.Panes = make([]protocol.PaneFrame, 0, len(window.PaneOrder))
	for _, index := range window.PaneOrder {
		rect, visible := rects[index]
		if !visible {
			continue
		}
		pane := window.Panes[index]
		frame.Panes = append(frame.Panes, protocol.PaneFrame{
			Index:  index,
			Rect:   rect,
			Cells:  terminalCells(pane, rect.W, rect.H),
			Active: index == window.ActivePane,
			Dead:   pane.Dead,
		})
	}
	frame.WindowHits = r.windowHits()
	return frame
}

func terminalCells(pane *Pane, width, height int) [][]protocol.Cell {
	rows := make([][]protocol.Cell, height)
	for y := range height {
		rows[y] = make([]protocol.Cell, width)
		for x := range width {
			rows[y][x] = protocolCell(pane.Term.CellAt(x, y))
		}
	}
	return rows
}

func protocolCell(cell *uv.Cell) protocol.Cell {
	if cell == nil {
		return protocol.Cell{Content: " ", Width: 1}
	}
	fg, hasFG := packedColor(cell.Style.Fg)
	bg, hasBG := packedColor(cell.Style.Bg)
	return protocol.Cell{
		Content: cell.Content,
		Width:   cell.Width,
		Style: protocol.CellStyle{
			Foreground:    fg,
			Background:    bg,
			HasForeground: hasFG,
			HasBackground: hasBG,
			Bold:          cell.Style.Attrs&uv.AttrBold != 0,
			Underline:     cell.Style.Underline != 0,
			Reverse:       cell.Style.Attrs&uv.AttrReverse != 0,
		},
	}
}

func packedColor(value color.Color) (uint32, bool) {
	if value == nil {
		return 0, false
	}
	red, green, blue, _ := value.RGBA()
	packed := uint32(red>>8)<<16 | uint32(green>>8)<<8 | uint32(blue>>8)
	return packed, true
}

func (r *Reactor) statusText() string {
	var out strings.Builder
	fmt.Fprintf(&out, "[%s] ", r.state.Name)
	for _, index := range r.state.WindowOrder {
		window := r.state.Windows[index]
		marker := "-"
		if index == r.state.ActiveWindow {
			marker = "*"
		}
		fmt.Fprintf(&out, "%d:%s%s ", index, window.Name, marker)
	}
	if window := r.state.Windows[r.state.ActiveWindow]; window != nil {
		if window.Synchronize {
			out.WriteString("[sync] ")
		}
		if window.Zoomed {
			out.WriteString("[zoom] ")
		}
	}
	out.WriteString(time.Now().Format("15:04:05"))
	return common.FitTerminalText(out.String(), r.state.Cols)
}

func (r *Reactor) windowHits() []protocol.WindowHit {
	start := len([]rune(fmt.Sprintf("[%s] ", r.state.Name)))
	hits := make([]protocol.WindowHit, 0, len(r.state.WindowOrder))
	for _, index := range r.state.WindowOrder {
		window := r.state.Windows[index]
		marker := "-"
		if index == r.state.ActiveWindow {
			marker = "*"
		}
		label := fmt.Sprintf("%d:%s%s ", index, window.Name, marker)
		end := start + len([]rune(label))
		hits = append(hits, protocol.WindowHit{
			Index: index,
			Start: start,
			End:   end,
		})
		start = end
	}
	return hits
}
