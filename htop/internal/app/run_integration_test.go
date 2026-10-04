package app

import (
	"bytes"
	"context"
	"runtime/pprof"
	"strings"
	"testing"
	"time"

	"github.com/gdamore/tcell/v3"
)

func TestInteractiveSessionRendersAndRespondsToKeyboardAndMouse(t *testing.T) {
	screen, result := startTestSession(t)

	frame := waitForFrame(t, screen, result, func(value string) bool {
		return strings.Contains(value, "Overview") && strings.Contains(value, "PID")
	})
	for _, want := range []string{"Overview", "Processes", "IO", "Network", "COMMAND"} {
		if !strings.Contains(frame, want) {
			t.Errorf("initial frame does not contain %q", want)
		}
	}

	sendTestEvent(screen, tcell.NewEventKey(tcell.KeyTab, "", tcell.ModNone))
	waitForFrame(t, screen, result, func(value string) bool {
		return !strings.Contains(value, "Host -  Uptime")
	})
	sendTestEvent(screen, tcell.NewEventKey(tcell.KeyTab, "", tcell.ModShift))
	waitForFrame(t, screen, result, func(value string) bool {
		return strings.Contains(value, "Host -  Uptime")
	})

	sendTestEvent(screen, tcell.NewEventKey(tcell.KeyF4, "", tcell.ModNone))
	sendTestEvent(screen, tcell.NewEventKey(tcell.KeyRune, "x", tcell.ModNone))
	waitForFrame(t, screen, result, func(value string) bool {
		return strings.Contains(value, "Filter (live): x_")
	})
	sendTestEvent(screen, tcell.NewEventKey(tcell.KeyEscape, "", tcell.ModNone))

	sendTestEvent(screen, tcell.NewEventMouse(27, 0, tcell.Button1, tcell.ModNone))
	sendTestEvent(screen, tcell.NewEventMouse(27, 0, tcell.ButtonNone, tcell.ModNone))
	waitForFrame(t, screen, result, func(value string) bool {
		return strings.Contains(value, "READ/s") && strings.Contains(value, "WRITE/s")
	})

	sendTestEvent(screen, tcell.NewEventKey(tcell.KeyRune, "q", tcell.ModNone))
	waitForExit(t, screen, result)
}

func TestInteractiveSessionCtrlCExitsAndFinalizesScreen(t *testing.T) {
	screen, result := startTestSession(t)
	waitForFrame(t, screen, result, func(value string) bool {
		return strings.Contains(value, "terman-htop")
	})

	sendTestEvent(screen, tcell.NewEventKey(tcell.KeyCtrlC, "c", tcell.ModCtrl))
	waitForExit(t, screen, result)
}

func startTestSession(t *testing.T) (*testScreen, <-chan error) {
	t.Helper()
	screen := newTestScreen(100, 30)
	result := make(chan error, 1)
	go func() {
		result <- runInteractive(
			context.Background(),
			Config{Refresh: time.Second, Sort: SortCPU},
			screen,
			idleSnapshotWorker,
		)
	}()
	return screen, result
}

func idleSnapshotWorker(ctx context.Context, _ time.Duration, _ chan Snapshot) {
	<-ctx.Done()
}

func sendTestEvent(screen *testScreen, event tcell.Event) {
	screen.EventQ() <- event
}

func waitForFrame(
	t *testing.T,
	screen *testScreen,
	result <-chan error,
	matches func(string) bool,
) string {
	t.Helper()
	deadline := time.Now().Add(2 * time.Second)
	for time.Now().Before(deadline) {
		select {
		case err := <-result:
			t.Fatalf("interactive session exited before expected frame: %v", err)
		default:
		}
		frame := testFrame(screen)
		if matches(frame) {
			return frame
		}
		time.Sleep(time.Millisecond)
	}
	t.Fatal("timed out waiting for expected frame")
	return ""
}

func waitForExit(t *testing.T, screen *testScreen, result <-chan error) {
	t.Helper()
	select {
	case err := <-result:
		if err != nil {
			t.Fatalf("interactive session returned an error: %v", err)
		}
	case <-time.After(2 * time.Second):
		var stacks bytes.Buffer
		_ = pprof.Lookup("goroutine").WriteTo(&stacks, 2)
		t.Fatalf("interactive session did not exit\n%s", stacks.String())
	}
	select {
	case <-screen.MouseDisabled():
	case <-time.After(2 * time.Second):
		t.Fatal("screen.DisableMouse was not called")
	}
	select {
	case <-screen.Finalized():
	case <-time.After(2 * time.Second):
		t.Fatal("screen.Fini was not called")
	}
}
