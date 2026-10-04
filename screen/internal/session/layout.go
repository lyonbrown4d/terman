package session

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/lyonbrown4d/terman/screen/internal/proto"
)

type region struct {
	windowID int64
	weight   int
}

func layout(regions []region, focused, cols, rows int, vertical bool) []proto.Region {
	if rows > 1 {
		rows--
	}
	total := 0
	for _, item := range regions {
		total += max(item.weight, 1)
	}
	result := make([]proto.Region, len(regions))
	used := 0
	position := 0
	dimension := rows
	if vertical {
		dimension = cols
	}
	for index, item := range regions {
		weight := max(item.weight, 1)
		nextUsed := used + weight
		next := dimension * nextUsed / max(total, 1)
		size := next - position
		if index == len(regions)-1 {
			size = dimension - position
		}
		x, y, width, height := 0, 0, cols, rows
		if len(regions) > 1 && vertical {
			x, width = position, max(size, 1)
		} else if len(regions) > 1 {
			y, height = position, max(size, 1)
		}
		result[index] = proto.Region{
			Index: index, X: x, Y: y, Width: max(width, 1),
			Height: max(height, 1), Focused: index == focused,
			Window: int(item.windowID),
		}
		used = nextUsed
		position = next
	}
	return result
}

func (o *Owner) resizeRegion(args []string) error {
	if len(o.regions) < 2 {
		return fmt.Errorf("no split region to resize")
	}
	spec := "="
	previous := false
	for _, arg := range args {
		switch arg {
		case "-h":
			if !o.vertical {
				return fmt.Errorf("no horizontal edge for focused region")
			}
		case "-v":
			if o.vertical {
				return fmt.Errorf("no vertical edge for focused region")
			}
		case "-b", "-l", "-p":
			previous = true
		default:
			spec = arg
		}
	}
	if spec == "=" {
		for index := range o.regions {
			o.regions[index].weight = 1000
		}
		o.resizeWindows()
		o.broadcast()
		return nil
	}
	total := 0
	for _, item := range o.regions {
		total += max(item.weight, 1)
	}
	neighbor := o.focused + 1
	if previous || neighbor >= len(o.regions) {
		neighbor = o.focused - 1
	}
	if neighbor < 0 {
		return fmt.Errorf("no region edge in requested direction")
	}
	if spec == "max" || spec == "min" {
		target := 1
		if spec == "max" {
			target = total - len(o.regions) + 1
		}
		o.setRegionWeight(neighbor, target)
		return nil
	}
	percent := strings.HasSuffix(spec, "%")
	raw := strings.TrimSuffix(spec, "%")
	sign := byte(0)
	if raw != "" && (raw[0] == '+' || raw[0] == '-') {
		sign, raw = raw[0], raw[1:]
	}
	value, err := strconv.Atoi(raw)
	if err != nil || value < 0 {
		return fmt.Errorf("invalid resize amount %q", spec)
	}
	dimension := o.rows - 1
	if o.vertical {
		dimension = o.cols
	}
	amount := value * total / max(dimension, 1)
	if percent {
		amount = value * total / 100
	}
	amount = max(amount, 1)
	current := max(o.regions[o.focused].weight, 1)
	delta := amount - current
	if sign == '+' {
		delta = amount
	} else if sign == '-' {
		delta = -amount
	}
	o.transferRegionWeight(neighbor, delta)
	return nil
}

func (o *Owner) setRegionWeight(neighbor, target int) {
	current := max(o.regions[o.focused].weight, 1)
	o.transferRegionWeight(neighbor, target-current)
}

func (o *Owner) transferRegionWeight(neighbor, delta int) {
	focusedWeight := max(o.regions[o.focused].weight, 1)
	neighborWeight := max(o.regions[neighbor].weight, 1)
	if delta > neighborWeight-1 {
		delta = neighborWeight - 1
	}
	if -delta > focusedWeight-1 {
		delta = -(focusedWeight - 1)
	}
	o.regions[o.focused].weight = focusedWeight + delta
	o.regions[neighbor].weight = neighborWeight - delta
	o.resizeWindows()
	o.broadcast()
}
