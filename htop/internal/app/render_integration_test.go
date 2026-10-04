package app

import (
	"strings"
	"testing"
)

func TestInitialDrawContainsTabsAndProcessHeader(t *testing.T) {
	screen := newTestScreen(100, 30)
	if err := screen.Init(); err != nil {
		t.Fatalf("initialize test screen: %v", err)
	}
	defer screen.Fini()

	state := newState(Config{Sort: SortCPU})
	state.draw(screen)
	frame := testFrame(screen)

	if strings.TrimSpace(frame) == "" {
		t.Fatal("initial frame is empty")
	}
	for _, want := range []string{"Overview", "Processes", "IO", "Network", "PID", "COMMAND"} {
		if !strings.Contains(frame, want) {
			t.Errorf("initial frame does not contain %q", want)
		}
	}
}
