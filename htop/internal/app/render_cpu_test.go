package app

import (
	"strings"
	"testing"
)

func TestOverviewDrawsEveryCPUCore(t *testing.T) {
	screen := newTestScreen(100, 30)
	if err := screen.Init(); err != nil {
		t.Fatalf("initialize test screen: %v", err)
	}
	defer screen.Fini()

	state := newState(Config{Sort: SortCPU})
	state.snapshot = Snapshot{
		CPU:         []float64{11.1, 22.2, 33.3, 44.4},
		MemoryUsed:  1,
		MemoryTotal: 2,
	}
	state.draw(screen)
	lines := strings.Split(testFrame(screen), "\n")

	assertLineContains(t, lines[2], "CPU0 [", "11.1%", "CPU2 [", "33.3%")
	assertLineContains(t, lines[3], "CPU1 [", "22.2%", "CPU3 [", "44.4%")
	assertLineContains(t, lines[4], "MEM", "50.0%", "1B/2B")
	assertLineContains(t, lines[5], "NET")
	assertLineContains(t, lines[6], "PID", "COMMAND")
}

func assertLineContains(t *testing.T, line string, values ...string) {
	t.Helper()
	for _, value := range values {
		if !strings.Contains(line, value) {
			t.Errorf("line %q does not contain %q", line, value)
		}
	}
}
