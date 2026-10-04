package app

import (
	"context"
	"strings"
	"testing"
	"time"

	"github.com/gdamore/tcell/v3"
)

func TestInteractiveSortMenuSupportsMouseSelection(t *testing.T) {
	screen := newTestScreen(100, 30)
	result := make(chan error, 1)
	worker := func(ctx context.Context, _ time.Duration, output chan Snapshot) {
		output <- Snapshot{
			CPU: []float64{10, 20},
			Processes: []Process{
				{PID: 1, Name: "cpu-first", CPU: 90, Memory: 1},
				{PID: 2, Name: "memory-first", CPU: 10, Memory: 4096},
			},
		}
		<-ctx.Done()
	}
	go func() {
		result <- runInteractive(
			context.Background(),
			Config{Refresh: time.Second, Sort: SortCPU},
			screen,
			worker,
		)
	}()

	waitForFrame(t, screen, result, func(value string) bool {
		cpu := strings.Index(value, "cpu-first")
		memory := strings.Index(value, "memory-first")
		return cpu >= 0 && memory >= 0 && cpu < memory
	})
	sendTestEvent(screen, tcell.NewEventKey(tcell.KeyF6, "", tcell.ModNone))
	waitForFrame(t, screen, result, func(value string) bool {
		return strings.Contains(value, "Sort by") && strings.Contains(value, "MEM")
	})

	width, height := screen.Size()
	panelWidth := min(52, max(24, width-4))
	visible := min(6, max(1, height-7))
	left := max(0, (width-panelWidth)/2)
	top := max(1, (height-(visible+3))/2)
	sendTestEvent(screen, tcell.NewEventMouse(left+2, top+2, tcell.Button1, tcell.ModNone))
	sendTestEvent(screen, tcell.NewEventMouse(left+2, top+2, tcell.ButtonNone, tcell.ModNone))

	frame := waitForFrame(t, screen, result, func(value string) bool {
		memory := strings.Index(value, "memory-first")
		cpu := strings.Index(value, "cpu-first")
		return strings.Contains(value, "MEMv") && memory >= 0 && cpu >= 0 && memory < cpu
	})
	if strings.Contains(frame, "Sort by") {
		t.Error("sort overlay remained open after mouse selection")
	}

	sendTestEvent(screen, tcell.NewEventKey(tcell.KeyRune, "q", tcell.ModNone))
	waitForExit(t, screen, result)
}
