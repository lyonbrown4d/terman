package app

import (
	"fmt"
	"strings"
)

func (s *state) drawStatus(c canvas) {
	y := c.height - 2
	c.row(y, styleBase)
	message := s.status
	style := styleMuted
	switch {
	case s.confirm != nil:
		message = fmt.Sprintf(
			"Send %s to %d process(es)? y confirms, n/Esc cancels",
			s.confirm.signal,
			len(s.confirm.pids),
		)
		style = styleDanger
	case s.input == inputFilter:
		message = "Filter (live): " + s.editor + "_"
		style = styleAccent
	case s.input == inputSearch:
		message = "Search: " + s.editor + "_"
		style = styleAccent
	case s.view != viewNone:
		message = "Up/Down/PgUp/PgDn scroll  Home/End jump  Esc/q closes"
	case message == "":
		parts := make([]string, 0, 4)
		if s.filter != "" {
			parts = append(parts, "filter="+s.filter)
		}
		if s.userFilter != "" {
			parts = append(parts, "user="+s.userFilter)
		}
		if s.followPID != 0 {
			parts = append(parts, fmt.Sprintf("follow=%d", s.followPID))
		}
		if len(s.tags) != 0 {
			parts = append(parts, fmt.Sprintf("tagged=%d", len(s.tags)))
		}
		message = strings.Join(parts, "  ")
		if message == "" {
			message = s.snapshot.Warning
			style = styleWarning
		}
	}
	c.text(0, y, style, fit(message, c.width))
}

func (s *state) drawFooter(c canvas) {
	y := c.height - 1
	c.row(y, styleHeader)
	buttons := []struct {
		label  string
		action string
	}{
		{label: "F2 Setup", action: "setup"},
		{label: "F3 Search", action: "search"},
		{label: "F4 Filter", action: "filter"},
		{label: "F5 Tree", action: "tree"},
		{label: "F6 Sort", action: "sort"},
		{label: "F7 Nice-", action: "nice-"},
		{label: "F8 Nice+", action: "nice+"},
		{label: "F9 Signal", action: "signal"},
		{label: "F10 Quit", action: "quit"},
	}
	x := 0
	for _, button := range buttons {
		label := " " + button.label + " "
		if x >= c.width {
			break
		}
		end := min(c.width, x+len(label))
		c.text(x, y, styleHeader, fit(label, end-x))
		s.hitboxes = append(s.hitboxes, hitbox{x1: x, y1: y, x2: end, y2: y + 1, action: button.action})
		x = end
	}
}
