//go:build integration

package e2e

import (
	"context"
	"fmt"
	"os"
	"strings"
	"testing"
	"time"

	"github.com/lyonbrown4d/terman/testkit/processtest"
)

func TestAbruptClientDisconnectAllowsReattach(t *testing.T) {
	h := newHarness(t)
	h.start("client-crash")
	client := h.attach("client-crash", false)
	ctx, cancel := context.WithTimeout(t.Context(), 5*time.Second)
	if !client.WaitFor(ctx, 20*time.Millisecond, readyText) {
		cancel()
		t.Fatalf("initial client did not render: %q", client.Output())
	}
	cancel()
	if err := client.Kill(); err != nil {
		t.Fatalf("kill client: %v", err)
	}
	ctx, cancel = context.WithTimeout(t.Context(), 5*time.Second)
	if !processtest.Wait(ctx, 25*time.Millisecond, func() bool {
		output, runErr := h.run("-S", h.session("client-crash"), "-X", "info")
		return runErr == nil && strings.Contains(output, "attach_clients=0")
	}) {
		cancel()
		t.Fatal("server did not remove the disconnected client")
	}
	cancel()

	second := h.attach("client-crash", false)
	ctx, cancel = context.WithTimeout(t.Context(), 5*time.Second)
	defer cancel()
	if !second.WaitFor(ctx, 20*time.Millisecond, readyText) {
		t.Fatalf("reattached client did not render: %q", second.Output())
	}
	h.detach(second)
}

func TestServerCrashAndStaleRecordRecovery(t *testing.T) {
	h := newHarness(t)
	record := h.start("server-crash")
	process, err := os.FindProcess(record.PID)
	if err != nil {
		t.Fatalf("find server process: %v", err)
	}
	if err := process.Kill(); err != nil {
		t.Fatalf("kill server process: %v", err)
	}

	ctx, cancel := context.WithTimeout(t.Context(), 10*time.Second)
	defer cancel()
	if !processtest.Wait(ctx, 50*time.Millisecond, func() bool {
		_, runErr := h.run("-S", h.session("server-crash"), "-X", "info")
		return runErr != nil
	}) {
		t.Fatal("control command still reached killed server")
	}
	if output, wipeErr := h.run("-wipe"); wipeErr != nil {
		t.Fatalf("wipe stale record: %v\n%s", wipeErr, output)
	}
	if _, found := h.find("server-crash"); found {
		t.Fatal("stale record remains after wipe")
	}

	restarted := h.start("server-crash")
	if restarted.PID == record.PID {
		t.Fatalf("expected a new server PID, still %d", record.PID)
	}
	if restarted.Endpoint == "" {
		t.Fatal(fmt.Errorf("restarted session has no endpoint"))
	}
}
