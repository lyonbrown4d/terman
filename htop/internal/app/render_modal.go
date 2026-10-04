package app

import (
	"fmt"
	"strings"
	"time"

	"github.com/gdamore/tcell/v3"
)

func (s *state) drawOverlay(c canvas) {
	overlay := s.overlay
	if overlay == nil {
		return
	}
	width := min(52, max(24, c.width-4))
	visible := min(len(overlay.entries), max(1, c.height-7))
	height := visible + 3
	left := max(0, (c.width-width)/2)
	top := max(1, (c.height-height)/2)
	panel := tcell.StyleDefault.Foreground(tcell.ColorWhite).Background(tcell.ColorDarkSlateGray)
	for y := top; y < top+height; y++ {
		c.screen.FillArea(left, y, min(width, c.width-left), 1, ' ', panel)
	}
	c.text(left+2, top, styleAccent.Background(tcell.ColorDarkSlateGray), fit(overlay.title, width-4))
	if overlay.selected < overlay.scroll {
		overlay.scroll = overlay.selected
	}
	if overlay.selected >= overlay.scroll+visible {
		overlay.scroll = overlay.selected - visible + 1
	}
	overlay.scroll = clamp(overlay.scroll, 0, max(0, len(overlay.entries)-visible))
	for row := 0; row < visible; row++ {
		index := overlay.scroll + row
		entry, ok := at(overlay.entries, index)
		if !ok {
			break
		}
		style := panel
		if index == overlay.selected {
			style = styleSelected
		}
		c.text(left+1, top+1+row, style, leftPad(entry.label, width-2))
	}
	c.text(left+2, top+height-1, panel, "Enter select  Esc close")
	s.hitboxes = append(s.hitboxes, hitbox{
		x1: left + 1, y1: top + 1, x2: left + width - 1, y2: top + 1 + visible, action: "overlay",
	})
}

func (s *state) drawFullView(c canvas) {
	process, alive := s.processByPID(s.viewPID)
	title := fmt.Sprintf(" Process detail: PID %d ", s.viewPID)
	lines := []string{"Process is no longer available."}
	if s.view == viewEnvironment {
		title = fmt.Sprintf(" Environment: PID %d ", s.viewPID)
		lines = s.environmentLines(c.width)
	} else if alive {
		lines = detailLines(process, c.width)
	}
	c.row(0, styleHeader)
	c.text(0, 0, styleHeader, fit(title, c.width))
	available := max(1, c.height-3)
	s.viewOffset = clamp(s.viewOffset, 0, max(0, len(lines)-available))
	for row := 0; row < available; row++ {
		index := s.viewOffset + row
		line, ok := at(lines, index)
		if !ok {
			break
		}
		c.text(0, row+1, styleBase, fit(line, c.width))
	}
}

func detailLines(process Process, width int) []string {
	started := "-"
	if process.Started > 0 {
		started = time.UnixMilli(process.Started).Format(time.RFC3339)
	}
	values := []string{
		fmt.Sprintf("PID: %d", process.PID),
		fmt.Sprintf("Parent PID: %d", process.PPID),
		"User: " + fallback(process.User, "-"),
		"State: " + fallback(process.Status, "-"),
		fmt.Sprintf("CPU: %.1f%%", process.CPU),
		"Resident memory: " + formatBytes(process.Memory),
		fmt.Sprintf("Nice: %d", process.Nice),
		"Started: " + started,
		"Read: " + formatBytes(process.ReadRate) + "/s  total " + formatBytes(process.ReadTotal),
		"Write: " + formatBytes(process.WriteRate) + "/s  total " + formatBytes(process.WriteTotal),
		"Name: " + fallback(process.Name, "-"),
		"Command: " + fallback(process.Command, "-"),
	}
	return wrapLines(values, width)
}

func (s *state) environmentLines(width int) []string {
	if len(s.environment) == 0 {
		return []string{"No environment variables were returned."}
	}
	return wrapLines(s.environment, width)
}

func wrapLines(values []string, width int) []string {
	width = max(1, width)
	lines := make([]string, 0, len(values))
	for _, value := range values {
		runes := []rune(value)
		if len(runes) == 0 {
			lines = append(lines, "")
			continue
		}
		for len(runes) > width {
			lines = append(lines, string(runes[:width]))
			runes = runes[width:]
		}
		lines = append(lines, string(runes))
	}
	return lines
}

func leftPad(value string, width int) string {
	value = fit(value, width)
	return value + strings.Repeat(" ", max(0, width-len([]rune(value))))
}
