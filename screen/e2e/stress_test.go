//go:build integration

package e2e

import (
	"context"
	"fmt"
	"sync"
	"testing"
	"time"

	"github.com/lyonbrown4d/terman/testkit/ptytest"
)

func TestFrequentAttachDetach(t *testing.T) {
	h := newHarness(t)
	h.start("attach-cycle")
	cycles := integerEnvironment("TERMAN_SCREEN_E2E_ATTACH_CYCLES", 12)
	for index := range cycles {
		client := h.attach("attach-cycle", false)
		ctx, cancel := context.WithTimeout(t.Context(), 5*time.Second)
		if !client.WaitFor(ctx, 20*time.Millisecond, readyText) {
			cancel()
			t.Fatalf("cycle %d did not render", index)
		}
		cancel()
		h.detach(client)
	}
}

func TestConcurrentMultiClientAttach(t *testing.T) {
	h := newHarness(t)
	h.start("multi-client")
	count := integerEnvironment("TERMAN_SCREEN_E2E_CLIENTS", 4)
	clients := make([]*ptytest.Client, count)
	for index := range count {
		clients[index] = h.attach("multi-client", true)
	}
	var failures sync.Map
	var workers sync.WaitGroup
	for index, client := range clients {
		workers.Go(func() {
			ctx, cancel := context.WithTimeout(t.Context(), 8*time.Second)
			defer cancel()
			if !client.WaitFor(ctx, 20*time.Millisecond, readyText) {
				failures.Store(index, fmt.Sprintf("client %d did not render", index))
				return
			}
			if err := client.Write([]byte(fmt.Sprintf("client-%d\r", index))); err != nil {
				failures.Store(index, err.Error())
			}
		})
	}
	workers.Wait()
	failures.Range(func(_, value any) bool {
		t.Error(value)
		return true
	})
	for _, client := range clients {
		h.detach(client)
	}
}

func TestLongRunningSession(t *testing.T) {
	h := newHarness(t)
	h.start("soak")
	duration := durationEnvironment("TERMAN_SCREEN_E2E_SOAK", 5*time.Second)
	deadline := time.Now().Add(duration)
	iteration := 0
	for time.Now().Before(deadline) {
		client := h.attach("soak", true)
		ctx, cancel := context.WithTimeout(t.Context(), 5*time.Second)
		if !client.WaitFor(ctx, 20*time.Millisecond, "\x1b[?1049h") {
			cancel()
			t.Fatalf("soak client %d did not enter alternate screen", iteration)
		}
		cancel()
		marker := fmt.Sprintf("soak-%d", iteration)
		if err := client.Write([]byte(marker + "\r")); err != nil {
			t.Fatalf("write soak client: %v", err)
		}
		ctx, cancel = context.WithTimeout(t.Context(), 5*time.Second)
		if !client.WaitFor(ctx, 20*time.Millisecond, "ECHO:"+marker) {
			cancel()
			t.Fatalf("soak client %d did not receive its echo", iteration)
		}
		cancel()
		h.detach(client)
		iteration++
	}
	if iteration == 0 {
		t.Fatal("soak test completed without an attach cycle")
	}
}
