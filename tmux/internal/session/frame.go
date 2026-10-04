package session

import (
	"fmt"
	"strings"
	"time"
	"unicode/utf8"

	"github.com/lyonbrown4d/terman/common"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
)

func (r *Reactor) frame() protocol.Frame {
	window := r.state.Windows[r.state.ActiveWindow]
	frame := protocol.Frame{
		Session: r.state.Name, Window: r.state.ActiveWindow,
		Status: r.statusText(), Message: r.state.Message, GeneratedAt: time.Now(),
	}
	if window == nil {
		return frame
	}
	frame.ActivePane = window.ActivePane
	rects := r.layoutRects(window)
	for _, index := range window.PaneOrder {
		rect, visible := rects[index]
		if !visible {
			continue
		}
		pane := window.Panes[index]
		lines := terminalLines(pane.Term.String(), rect.W, rect.H)
		frame.Panes = append(frame.Panes, protocol.PaneFrame{
			Index: index, Rect: rect, Lines: lines, Active: index == window.ActivePane, Dead: pane.Dead,
		})
	}
	frame.WindowHits = r.windowHits()
	return frame
}

func terminalLines(value string, width, height int) []string {
	value = strings.ReplaceAll(value, "\r\n", "\n")
	value = strings.ReplaceAll(value, "\r", "")
	lines := strings.Split(value, "\n")
	if len(lines) > height {
		lines = lines[len(lines)-height:]
	}
	result := make([]string, 0, height)
	for _, line := range lines {
		result = append(result, common.FitTerminalText(stripControls(line), width))
	}
	for len(result) < height {
		result = append(result, strings.Repeat(" ", max(width, 0)))
	}
	return result
}

func stripControls(value string) string {
	var out strings.Builder
	escaped := false
	for len(value) > 0 {
		char, size := utf8.DecodeRuneInString(value)
		value = value[size:]
		if char == '\x1b' {
			escaped = true
			continue
		}
		if escaped {
			if (char >= '@' && char <= '~') || char == '\a' {
				escaped = false
			}
			continue
		}
		if char >= 0x20 || char == '\t' {
			out.WriteRune(char)
		}
	}
	return out.String()
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
		hits = append(hits, protocol.WindowHit{Index: index, Start: start, End: end})
		start = end
	}
	return hits
}
