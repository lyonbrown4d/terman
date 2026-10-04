package session

import "github.com/lyonbrown4d/terman/screen/internal/proto"

type region struct {
	windowID int64
}

func layout(regions []region, focused, cols, rows int, vertical bool) []proto.Region {
	if rows > 1 {
		rows--
	}
	result := make([]proto.Region, len(regions))
	for index, item := range regions {
		x, y, width, height := 0, 0, cols, rows
		if len(regions) > 1 && vertical {
			x = cols * index / len(regions)
			next := cols * (index + 1) / len(regions)
			width = next - x
		} else if len(regions) > 1 {
			y = rows * index / len(regions)
			next := rows * (index + 1) / len(regions)
			height = next - y
		}
		result[index] = proto.Region{
			Index: index, X: x, Y: y, Width: max(width, 1),
			Height: max(height, 1), Focused: index == focused,
			Window: int(item.windowID),
		}
	}
	return result
}
