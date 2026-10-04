package app

import (
	"strconv"
)

const cpuMeterMinimumWidth = 24

func (s *state) drawCPUMeters(c canvas, startY int) int {
	values := s.snapshot.CPU
	if len(values) == 0 {
		values = []float64{0}
	}

	columns := cpuMeterColumns(c, len(values), startY)
	rows := (len(values) + columns - 1) / columns
	columnWidth := c.width / columns
	for column := range columns {
		for row := range rows {
			index := column*rows + row
			if index >= len(values) {
				break
			}
			x := column * columnWidth
			width := columnWidth
			if column == columns-1 {
				width = c.width - x
			}
			label := "CPU" + strconv.Itoa(index)
			c.text(x, startY+row, styleBase, fit(meter(label, values[index], 100, width), width))
		}
	}
	return startY + rows
}

func cpuMeterColumns(c canvas, cores, startY int) int {
	maxColumns := max(1, c.width/cpuMeterMinimumWidth)
	maxRows := max(1, c.height-startY-6)
	columns := 1
	if cores > 1 && maxColumns > 1 {
		columns = 2
	}
	needed := (cores + maxRows - 1) / maxRows
	columns = max(columns, needed)
	return clamp(columns, 1, min(cores, maxColumns))
}
