package common

import (
	"math"
	"strings"

	"github.com/mattn/go-runewidth"
)

// TerminalTextWidth returns the number of terminal cells occupied by text.
// The result saturates at the largest uint16 value.
func TerminalTextWidth(text string) uint16 {
	width := runewidth.StringWidth(text)
	if width > math.MaxUint16 {
		return math.MaxUint16
	}
	return uint16(width)
}

// FitTerminalText truncates text to width cells and pads the remainder with spaces.
func FitTerminalText(text string, width int) string {
	if width <= 0 {
		return ""
	}

	var output strings.Builder
	used := 0
	for _, char := range text {
		next := used + runewidth.RuneWidth(char)
		if next > width {
			break
		}
		output.WriteRune(char)
		used = next
	}
	output.WriteString(strings.Repeat(" ", width-used))
	return output.String()
}

// TruncateTerminalText truncates text to width cells and appends three dots.
// As in the Rust implementation, widths below three still produce "...".
func TruncateTerminalText(text string, width int) string {
	if int(TerminalTextWidth(text)) <= width {
		return text
	}

	bodyWidth := max(width-3, 0)
	var output strings.Builder
	used := 0
	for _, char := range text {
		next := used + runewidth.RuneWidth(char)
		if next > bodyWidth {
			break
		}
		output.WriteRune(char)
		used = next
	}
	output.WriteString("...")
	return output.String()
}
