//go:build integration

package e2e

import (
	"context"
	"runtime"
	"strings"
	"testing"
	"time"
)

func TestRealBinaryPTYAndNativeIPC(t *testing.T) {
	h := newHarness(t)
	record := h.start("native-ipc")
	if runtime.GOOS == "windows" {
		if !strings.HasPrefix(record.Endpoint, `\\.\pipe\terman-screen-`) {
			t.Fatalf("expected named pipe endpoint, got %q", record.Endpoint)
		}
	} else if !strings.HasSuffix(record.Endpoint, ".sock") {
		t.Fatalf("expected Unix socket endpoint, got %q", record.Endpoint)
	}

	client := h.attach("native-ipc", false)
	if err := client.Resize(120, 40); err != nil {
		t.Fatalf("resize real PTY: %v", err)
	}
	ctx, cancel := context.WithTimeout(t.Context(), 5*time.Second)
	defer cancel()
	if !client.WaitFor(ctx, 25*time.Millisecond, readyText) {
		t.Fatalf("session output not rendered through PTY: %q", client.Output())
	}
	if err := client.Write([]byte("hello-real-pty\r")); err != nil {
		t.Fatalf("write real PTY: %v", err)
	}
	h.detach(client)
	if _, found := h.find("native-ipc"); !found {
		t.Fatal("detaching the client stopped the server")
	}
}

func TestControlCommandsUseLiveServer(t *testing.T) {
	h := newHarness(t)
	h.start("control")
	commands := [][]string{
		{"version"},
		{"number"},
		{"width", "100", "30"},
		{"height", "35"},
		{"windows"},
		{"title", "renamed"},
		{"split"},
		{"focus"},
		{"resize", "10"},
		{"only"},
		{"scrollback", "2000"},
	}
	for _, command := range commands {
		t.Run(strings.Join(command, "-"), func(t *testing.T) {
			h.control("control", command...)
		})
	}
}
