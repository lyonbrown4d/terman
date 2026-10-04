package app

import "github.com/gdamore/tcell/v2"

var (
	styleBase     = tcell.StyleDefault.Foreground(tcell.ColorSilver).Background(tcell.ColorBlack)
	styleAccent   = tcell.StyleDefault.Foreground(tcell.ColorAqua).Bold(true)
	styleHeader   = tcell.StyleDefault.Foreground(tcell.ColorBlack).Background(tcell.ColorTeal).Bold(true)
	styleSelected = tcell.StyleDefault.Foreground(tcell.ColorBlack).Background(tcell.ColorGreen)
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
			c.screen.SetContent(x, y, character, nil, style)
		}
		x++
	}
}

func (c canvas) row(y int, style tcell.Style) {
	if y < 0 || y >= c.height {
		return
	}
	for x := 0; x < c.width; x++ {
		c.screen.SetContent(x, y, ' ', nil, style)
	}
}

func (s *state) draw(screen tcell.Screen) {
	screen.Clear()
	c := newCanvas(screen)
	if c.width < 30 || c.height < 8 {
		c.text(0, 0, styleWarning, "Terminal too small (minimum 30x8)")
		screen.Show()
		return
	}
	s.drawTabs(c)
	switch s.tab {
	case TabOverview:
		s.drawOverview(c)
	case TabProcesses:
		s.drawProcesses(c)
	case TabIO:
		s.drawIO(c)
	case TabNetwork:
		s.drawNetwork(c)
	}
	if s.detail {
		s.drawDetail(c)
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
		x += len(label)
	}
	title := " terman-htop "
	c.text(max(x, c.width-len(title)), 0, styleAccent, title)
}

func (s *state) drawOverview(c canvas) {
	c.text(0, 1, styleAccent, fit("Host "+fallback(s.snapshot.Hostname, "-")+"  Uptime "+formatDuration(s.snapshot.Uptime), c.width))
	c.text(0, 2, styleBase, meter("CPU", cpuAverage(s.snapshot.CPU), 100, c.width))
	c.text(0, 3, styleBase, meter("MEM", percent(s.snapshot.MemoryUsed, s.snapshot.MemoryTotal), 100, c.width))
	var sent, received uint64
	for _, row := range s.snapshot.Interfaces {
		sent += row.SendRate
		received += row.RecvRate
	}
	c.text(0, 4, styleBase, fit("NET  RX/s "+formatBytes(received)+"  TX/s "+formatBytes(sent), c.width))
	s.drawProcessTable(c, 5)
}

func (s *state) drawProcesses(c canvas) {
	s.drawProcessTable(c, 1)
}

func (s *state) drawProcessTable(c canvas, headerY int) {
	c.row(headerY, styleHeader)
	c.text(0, headerY, styleHeader, processHeader(c.width))
	rows := processRows(s.snapshot, s.sort, s.reverse, s.filter)
	start, end := s.visibleRange(c, len(rows), headerY)
	for index := start; index < end; index++ {
		row := rows[index]
		style := styleBase
		if index == s.selected[s.tab] {
			style = styleSelected
		}
		c.row(headerY+1+index-start, style)
		c.text(0, headerY+1+index-start, style, processLine(row, c.width))
	}
}

func processHeader(width int) string {
	return fit("PID     USER        CPU%    MEM       IO/s      S NAME / COMMAND", width)
}

func processLine(row Process, width int) string {
	value := formatColumns(
		[]string{formatInt(row.PID, 7), fit(row.User, 11), formatFloat(row.CPU, 7),
			left(formatBytes(row.Memory), 10), left(formatBytes(row.ReadRate+row.WriteRate), 10),
			fallback(row.Name, "-") + " " + row.Command},
		[]int{7, 12, 8, 10, 10, max(1, width-47)},
	)
	return fit(value, width)
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
		if index == s.selected[s.tab] {
			style = styleSelected
		}
		line := formatColumns(
			[]string{formatInt(row.PID, 7), formatBytes(row.ReadRate), formatBytes(row.WriteRate),
				formatBytes(row.ReadTotal), formatBytes(row.WriteTotal), row.Name},
			[]int{7, 11, 11, 13, 13, max(1, c.width-55)},
		)
		c.row(headerY+1+index-start, style)
		c.text(0, headerY+1+index-start, style, fit(line, c.width))
	}
}

func (s *state) drawNetwork(c canvas) {
	c.text(0, 1, styleAccent, "Interfaces")
	y := 2
	limit := min(len(s.snapshot.Interfaces), 3)
	for _, row := range s.snapshot.Interfaces[:limit] {
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
		if index == s.selected[s.tab] {
			style = styleSelected
		}
		line := formatColumns(
			[]string{row.Protocol, formatInt(row.PID, 7), row.Local, row.Remote, row.Status, row.Process},
			[]int{6, 8, 26, 26, 13, max(1, c.width-79)},
		)
		c.row(y+1+index-start, style)
		c.text(0, y+1+index-start, style, fit(line, c.width))
	}
}

func (s *state) drawDetail(c canvas) {
	process, ok := s.currentProcess()
	if !ok {
		return
	}
	top := max(s.bodyStart()+2, c.height-10)
	detailStyle := tcell.StyleDefault.Background(tcell.ColorDarkSlateGray).Foreground(tcell.ColorWhite)
	for y := top; y < c.height-2; y++ {
		c.row(y, detailStyle)
	}
	c.text(0, top, styleAccent.Background(tcell.ColorDarkSlateGray), " Process detail ")
	lines := []string{
		"PID " + formatInt(process.PID, 0) + "  PPID " + formatInt(process.PPID, 0) + "  USER " + process.User + "  STATE " + process.Status,
		"CPU " + formatFloat(process.CPU, 0) + "%  RSS " + formatBytes(process.Memory) + "  NICE " + formatInt(process.Nice, 0),
		"READ " + formatBytes(process.ReadRate) + "/s (" + formatBytes(process.ReadTotal) + ")  WRITE " + formatBytes(process.WriteRate) + "/s (" + formatBytes(process.WriteTotal) + ")",
		"NAME " + process.Name,
		"CMD  " + process.Command,
	}
	for index, line := range lines {
		if top+1+index < c.height-2 {
			c.text(0, top+1+index, detailStyle, fit(line, c.width))
		}
	}
	if len(s.environment) != 0 && top+6 < c.height-2 {
		c.text(0, top+6, detailStyle, fit("ENV  "+joinEnvironment(s.environment), c.width))
	}
}
