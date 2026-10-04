package cli

import (
	"encoding/json"
	"fmt"

	"github.com/lyonbrown4d/terman/common"
)

func terminalSize() (int, int) {
	cols, rows, err := common.CurrentTerminalSize()
	if err != nil || cols < 2 || rows < 2 {
		return settings.TerminalCols, settings.TerminalRows
	}
	return int(cols), int(rows)
}

func printJSON(value any) error {
	data, err := json.MarshalIndent(value, "", "  ")
	if err != nil {
		return err
	}
	fmt.Println(string(data))
	return nil
}
