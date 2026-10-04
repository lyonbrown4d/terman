package app

import (
	"fmt"
	"math"
	"strconv"
	"strings"
	"time"

	"github.com/gdamore/tcell/v2"
)

func (s *state) drawStatus(c canvas) {
	y := c.height - 2
	c.row(y, styleBase)
	message := s.status
	style := styleMuted
	if s.confirm != "" {
		process, _ := s.currentProcess()
		message = fmt.Sprintf("%s PID %d? press y to confirm, n to cancel", strings.ToUpper(s.confirm), process.PID)
		style = styleDanger
	} else if s.filterEdit {
		message = "Filter: " + s.filter + "_"
		style = styleAccent
	} else if message == "" {
		message = s.snapshot.Warning
		style = styleWarning
	}
	c.text(0, y, style, fit(message, c.width))
}

func (s *state) drawFooter(c canvas) {
	y := c.height - 1
	c.row(y, styleHeader)
	direction := "normal"
	if s.reverse {
		direction = "reverse"
	}
	value := fmt.Sprintf(" Arrows/Mouse select  / filter  %s sort:%s  r:%s  d detail  e env  [ ] nice  k/K signal  F10 quit ",
		fallback(s.filter, "-"), s.sort.String(), direction)
	c.text(0, y, styleHeader, fit(value, c.width))
}

func (s *state) visibleRange(c canvas, total, headerY int) (int, int) {
	rows := c.height - headerY - 3
	if s.detail {
		rows -= 8
	}
	rows = max(0, rows)
	selected := clamp(s.selected[s.tab], 0, max(0, total-1))
	s.selected[s.tab] = selected
	scroll := clamp(s.scroll[s.tab], 0, max(0, total-rows))
	if selected < scroll {
		scroll = selected
	}
	if selected >= scroll+rows && rows > 0 {
		scroll = selected - rows + 1
	}
	s.scroll[s.tab] = scroll
	return scroll, min(total, scroll+rows)
}

func (s *state) normalize(screen tcell.Screen) {
	total := s.currentLength()
	s.selected[s.tab] = clamp(s.selected[s.tab], 0, max(0, total-1))
	_, height := screen.Size()
	rows := max(0, height-s.bodyStart()-3)
	if s.detail {
		rows = max(0, rows-8)
	}
	s.scroll[s.tab] = clamp(s.scroll[s.tab], 0, max(0, total-rows))
}

func (s *state) currentLength() int {
	switch s.tab {
	case TabOverview, TabProcesses:
		return len(processRows(s.snapshot, s.sort, s.reverse, s.filter))
	case TabIO:
		return len(s.snapshot.IO)
	default:
		return len(s.snapshot.Connections)
	}
}

func (s *state) bodyStart() int {
	switch s.tab {
	case TabOverview:
		return 5
	case TabNetwork:
		return 2 + min(len(s.snapshot.Interfaces), 3)
	default:
		return 1
	}
}

func tabAt(x int) (Tab, bool) {
	offset := 0
	for index, name := range tabNames {
		width := len(name) + 5
		if x >= offset && x < offset+width {
			return Tab(index), true
		}
		offset += width
	}
	return 0, false
}

func (s *state) setStatus(value string) {
	s.status, s.statusAt = value, time.Now()
}

func formatBytes(bytes uint64) string {
	const unit = 1024
	if bytes < unit {
		return strconv.FormatUint(bytes, 10) + "B"
	}
	divisor, exponent := uint64(unit), 0
	for quotient := bytes / unit; quotient >= unit && exponent < 4; quotient /= unit {
		divisor *= unit
		exponent++
	}
	return fmt.Sprintf("%.1f%ciB", float64(bytes)/float64(divisor), "KMGTPE"[exponent])
}

func formatDuration(seconds uint64) string {
	days := seconds / 86400
	hours := seconds % 86400 / 3600
	minutes := seconds % 3600 / 60
	if days != 0 {
		return fmt.Sprintf("%dd %02dh %02dm", days, hours, minutes)
	}
	return fmt.Sprintf("%02dh %02dm", hours, minutes)
}

func meter(label string, value, maximum float64, width int) string {
	barWidth := clamp(width-24, 6, 60)
	filled := 0
	if maximum > 0 {
		filled = clamp(int(math.Round(value/maximum*float64(barWidth))), 0, barWidth)
	}
	return fmt.Sprintf("%-4s [%s%s] %6.1f%%", label, strings.Repeat("|", filled), strings.Repeat(" ", barWidth-filled), value)
}

func cpuAverage(values []float64) float64 {
	if len(values) == 0 {
		return 0
	}
	var sum float64
	for _, value := range values {
		sum += value
	}
	return sum / float64(len(values))
}

func percent(value, total uint64) float64 {
	if total == 0 {
		return 0
	}
	return float64(value) / float64(total) * 100
}

func formatColumns(values []string, widths []int) string {
	var builder strings.Builder
	for index, value := range values {
		if index != 0 {
			builder.WriteByte(' ')
		}
		builder.WriteString(left(value, widths[index]))
	}
	return builder.String()
}

func left(value string, width int) string {
	runes := []rune(value)
	if len(runes) > width {
		if width <= 1 {
			return string(runes[:max(0, width)])
		}
		return string(runes[:width-1]) + "~"
	}
	return value + strings.Repeat(" ", max(0, width-len(runes)))
}

func fit(value string, width int) string {
	return strings.TrimRight(left(value, max(0, width)), " ")
}

func fallback(value, replacement string) string {
	if value == "" {
		return replacement
	}
	return value
}

func formatFloat(value float64, width int) string {
	if width == 0 {
		return fmt.Sprintf("%.1f", value)
	}
	return fmt.Sprintf("%*.1f", width, value)
}

func formatInt[T ~int32 | ~int64](value T, width int) string {
	if width == 0 {
		return strconv.FormatInt(int64(value), 10)
	}
	return fmt.Sprintf("%*d", width, value)
}

func joinEnvironment(values []string) string {
	const limit = 6
	if len(values) > limit {
		return strings.Join(values[:limit], "; ") + fmt.Sprintf("; ... (%d more)", len(values)-limit)
	}
	return strings.Join(values, "; ")
}

func at[T any](values []T, index int) (T, bool) {
	var zero T
	if index < 0 || index >= len(values) {
		return zero, false
	}
	return values[index], true
}

func clamp(value, low, high int) int {
	if high < low {
		return low
	}
	return min(max(value, low), high)
}
