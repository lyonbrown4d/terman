package common

import (
	"os"

	"golang.org/x/term"
)

// TerminalRowsWithoutStatus reserves one row for a status line and keeps at least one content row.
func TerminalRowsWithoutStatus(rows uint16) uint16 {
	if rows <= 1 {
		return 1
	}
	return rows - 1
}

// IsTerminalLastRow reports whether row is the terminal's zero-based last row.
func IsTerminalLastRow(row, rows uint16) bool {
	last := uint16(0)
	if rows > 0 {
		last = rows - 1
	}
	return row == last
}

// IsCurrentTerminalLastRow queries stdout and checks row against its height.
func IsCurrentTerminalLastRow(row uint16) bool {
	_, rows, err := CurrentTerminalSize()
	return err == nil && IsTerminalLastRow(row, rows)
}

// CurrentTerminalSize returns stdout's columns and rows.
func CurrentTerminalSize() (columns, rows uint16, err error) {
	width, height, err := term.GetSize(int(os.Stdout.Fd()))
	if err != nil {
		return 0, 0, err
	}
	return saturatingUint16(width), saturatingUint16(height), nil
}

func saturatingUint16(value int) uint16 {
	if value <= 0 {
		return 0
	}
	if value > int(^uint16(0)) {
		return ^uint16(0)
	}
	return uint16(value)
}
