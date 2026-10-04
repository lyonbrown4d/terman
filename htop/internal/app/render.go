package app

import (
	"fmt"
	"strings"

	"github.com/gdamore/tcell/v3"
)

var (
	styleBase     = tcell.StyleDefault.Foreground(tcell.ColorSilver).Background(tcell.ColorBlack)
	styleAccent   = tcell.StyleDefault.Foreground(tcell.ColorAqua).Bold(true)
	styleHeader   = tcell.StyleDefault.Foreground(tcell.ColorBlack).Background(tcell.ColorTeal).Bold(true)
	styleSelected = tcell.StyleDefault.Foreground(tcell.ColorBlack).Background(tcell.ColorGreen)
	styleTagged   = tcell.StyleDefault.Foreground(tcell.ColorBlack).Background(tcell.ColorDarkCyan)
	styleMuted    = tcell.StyleDefault.Foreground(tcell.ColorGray)
	styleWarning  = tcell.StyleDefault.Foreground(tcell.ColorYellow)
	styleDanger   = tcell.StyleDefault.Foreground(tcell.ColorRed).Bold(true)
)

type canvas struct {
	screen tcell.Screen
	width  int
	height int
}

func newCanvas(screen tcell.Screen) canvas {
	width, height := screen.Size()
	return canvas{screen: screen, width: width, height: height}
}

func (c canvas) text(x, y int, style tcell.Style, value string) {
	if y < 0 || y >= c.height || x >= c.width {
		return
	}
	for _, character := range []rune(value) {
		if x >= c.width {
			return
		}
		if x >= 0 {
			_, cellWidth := c.screen.Put(x, y, string(character), style)
			if cellWidth > 1 {
				x += cellWidth - 1
			}
		}
		x++
	}
}

func (c canvas) row(y int, style tcell.Style) {
	if y < 0 || y >= c.height {
		return
	}
	c.screen.FillArea(0, y, c.width, 1, ' ', style)
}

func (s *state) draw(screen tcell.Screen) {
	screen.Clear()
	c := newCanvas(screen)
	s.hitboxes = s.hitboxes[:0]
	if c.width < 30 || c.height < 8 {
		c.text(0, 0, styleWarning, "Terminal too small (minimum 30x8)")
		screen.Show()
		return
	}
	if s.view != viewNone {
		s.drawFullView(c)
	} else {
		s.drawTabs(c)
		s.drawCurrentTab(c)
		if s.overlay != nil {
			s.drawOverlay(c)
		}
	}
	s.drawStatus(c)
	s.drawFooter(c)
	screen.Show()
}

func (s *state) drawTabs(c canvas) {
	x := 0
	for index, name := range tabNames {
		label := " " + string(rune('1'+index)) + " " + name + " "
		style := styleMuted
		if Tab(index) == s.tab {
			style = styleHeader
		}
		c.text(x, 0, style, label)
		s.hitboxes = append(s.hitboxes, hitbox{x1: x, y1: 0, x2: x + len(label), y2: 1, action: fmt.Sprintf("tab:%d", index)})
		x += len(label)
	}
	title := " terman-htop "
	c.text(max(x, c.width-len(title)), 0, styleAccent, title)
}

func (s *state) drawCurrentTab(c canvas) {
	switch s.tab {
	case TabOverview:
		s.drawOverview(c)
	case TabProcesses:
		s.drawProcessTable(c, 1)
	case TabIO:
		s.drawIO(c)
	case TabNetwork:
		s.drawNetwork(c)
	}
}

func (s *state) drawOverview(c canvas) {
	heading := "Host " + fallback(s.snapshot.Hostname, "-") + "  Uptime " + formatDuration(s.snapshot.Uptime)
	c.text(0, 1, styleAccent, fit(heading, c.width))
	nextY := s.drawCPUMeters(c, 2)
	memory := formatBytes(s.snapshot.MemoryUsed) + "/" + formatBytes(s.snapshot.MemoryTotal)
	c.text(
		0,
		nextY,
		styleBase,
		meterDetails("MEM", percent(s.snapshot.MemoryUsed, s.snapshot.MemoryTotal), 100, c.width, memory),
	)
	var sent, received uint64
	for _, row := range s.snapshot.Interfaces {
		sent += row.SendRate
		received += row.RecvRate
	}
	nextY++
	c.text(0, nextY, styleBase, fit("NET  RX/s "+formatBytes(received)+"  TX/s "+formatBytes(sent), c.width))
	s.drawProcessTable(c, nextY+1)
}

func (s *state) drawProcessTable(c canvas, headerY int) {
	if s.tab == TabOverview {
		s.processHeader = headerY
	}
	c.row(headerY, styleHeader)
	c.text(0, headerY, styleHeader, s.processHeaderLine(c.width))
	s.addProcessHeaderHitboxes(headerY, c.width)
	rows := s.visibleProcessRows()
	start, end := s.visibleRange(c, len(rows), headerY)
	for index := start; index < end; index++ {
		row := rows[index]
		style := styleBase
		if s.tags[row.PID] {
			style = styleTagged
		}
		if index == s.selected[s.tab] {
			style = styleSelected
		}
		y := headerY + 1 + index - start
		c.row(y, style)
		c.text(0, y, style, s.processLine(row, c.width))
	}
}

func (s *state) processLine(row ProcessRow, width int) string {
	tag := " "
	if s.tags[row.PID] {
		tag = "*"
	}
	name := fallback(row.Name, "-")
	if s.showCommand && row.Command != "" {
		name += " " + row.Command
	}
	if s.tree {
		branch := "  "
		if row.HasChildren && row.Collapsed {
			branch = "+ "
		} else if row.HasChildren {
			branch = "- "
		}
		name = strings.Repeat("| ", row.Depth) + branch + name
	}
	value := formatColumns(
		[]string{tag, formatInt(row.PID, 7), row.User, formatFloat(row.CPU, 7),
			formatBytes(row.Memory), formatBytes(row.ReadRate + row.WriteRate), row.Status, name},
		[]int{1, 7, 12, 8, 10, 10, 2, max(1, width-57)},
	)
	return fit(value, width)
}

func (s *state) addProcessHeaderHitboxes(y, width int) {
	boxes := []hitbox{
		{x1: 0, y1: y, x2: 10, y2: y + 1, action: "sort-pid"},
		{x1: 10, y1: y, x2: 23, y2: y + 1, action: "sort-user"},
		{x1: 23, y1: y, x2: 32, y2: y + 1, action: "sort-cpu"},
		{x1: 32, y1: y, x2: 43, y2: y + 1, action: "sort-memory"},
		{x1: 43, y1: y, x2: 54, y2: y + 1, action: "sort-io"},
		{x1: 54, y1: y, x2: width, y2: y + 1, action: "sort-name"},
	}
	s.hitboxes = append(s.hitboxes, boxes...)
}

func (s *state) drawIO(c canvas) {
	headerY := 1
	c.row(headerY, styleHeader)
	c.text(0, headerY, styleHeader, fit("PID     READ/s     WRITE/s    TOTAL READ   TOTAL WRITE  NAME", c.width))
	rows := ioRows(s.snapshot, s.reverse)
	start, end := s.visibleRange(c, len(rows), headerY)
	for index := start; index < end; index++ {
		row := rows[index]
		style := styleBase
		if s.tags[row.PID] {
			style = styleTagged
		}
		if index == s.selected[s.tab] {
			style = styleSelected
		}
		line := formatColumns(
			[]string{formatInt(row.PID, 7), formatBytes(row.ReadRate), formatBytes(row.WriteRate),
				formatBytes(row.ReadTotal), formatBytes(row.WriteTotal), row.Name},
			[]int{7, 11, 11, 13, 13, max(1, c.width-60)},
		)
		y := headerY + 1 + index - start
		c.row(y, style)
		c.text(0, y, style, fit(line, c.width))
	}
}

func (s *state) drawNetwork(c canvas) {
	c.text(0, 1, styleAccent, "Interfaces")
	y := 2
	for _, row := range s.snapshot.Interfaces[:min(len(s.snapshot.Interfaces), 3)] {
		line := formatColumns(
			[]string{row.Name, "RX/s " + formatBytes(row.RecvRate), "TX/s " + formatBytes(row.SendRate)},
			[]int{18, 18, 18},
		)
		c.text(0, y, styleBase, fit(line, c.width))
		y++
	}
	c.row(y, styleHeader)
	c.text(0, y, styleHeader, fit("PROTO PID     LOCAL                     REMOTE                    STATE PROCESS", c.width))
	start, end := s.visibleRange(c, len(s.snapshot.Connections), y)
	for index := start; index < end; index++ {
		row := s.snapshot.Connections[index]
		style := styleBase
		if s.tags[row.PID] {
			style = styleTagged
		}
		if index == s.selected[s.tab] {
			style = styleSelected
		}
		line := formatColumns(
			[]string{row.Protocol, formatInt(row.PID, 7), row.Local, row.Remote, row.Status, row.Process},
			[]int{6, 8, 26, 26, 13, max(1, c.width-84)},
		)
		rowY := y + 1 + index - start
		c.row(rowY, style)
		c.text(0, rowY, style, fit(line, c.width))
	}
}
