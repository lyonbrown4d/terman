//go:build integration

package service

import (
	"context"
	"os"
	"path/filepath"
	"testing"
	"time"
)

func TestBrokerReapsNaturalExitsAndGracefulShutdown(t *testing.T) {
	t.Setenv("TERMAN_SERVICE_TEST_CHILD", "1")
	marker := filepath.Join(t.TempDir(), "child")
	cfg := Config{Tool: "natural-test-" + filepath.Base(t.TempDir()), ServerArgument: "--service-child"}
	cfg.Shutdown = func(_ context.Context, launch Launch) error { return os.WriteFile(launch.Args[0]+".stop", nil, 0600) }
	b, err := newBroker(cfg)
	if err != nil {
		t.Fatal(err)
	}
	if err = b.Start(nil); err != nil {
		t.Fatal(err)
	}
	defer b.Stop(nil)
	_, used, err := TryStart(t.Context(), cfg, Launch{Name: "exit", Cwd: filepath.Dir(marker), Args: []string{marker, "exit"}})
	if !used || err != nil {
		t.Fatalf("start natural exit: %v %v", used, err)
	}
	waitFor(t, func() bool { return b.status().Sessions == 0 }, "natural child to be reaped")
	pid, used, err := TryStart(t.Context(), cfg, Launch{Name: "graceful", Cwd: filepath.Dir(marker), Args: []string{marker}})
	if !used || err != nil || pid <= 0 {
		t.Fatalf("start graceful: %d %v %v", pid, used, err)
	}
	waitFor(t, func() bool { _, err := os.Stat(marker); return err == nil }, "child marker")
	if err = b.Stop(nil); err != nil {
		t.Fatal(err)
	}
	if b.status().Sessions != 0 {
		t.Fatal("stopped broker retained child processes")
	}
	if _, err = os.Stat(marker + ".stop"); err != nil {
		t.Fatal("graceful shutdown callback did not run")
	}
}

func TestStoppingOneInstanceLeavesAnotherRunning(t *testing.T) {
	cfg := Config{Tool: "isolation-test-" + filepath.Base(t.TempDir()), ServerArgument: "--server"}
	first, err := newBroker(cfg)
	if err != nil {
		t.Fatal(err)
	}
	if err = first.Start(nil); err != nil {
		t.Fatal(err)
	}
	defer first.Stop(nil)
	otherCfg := cfg
	otherCfg.ServiceName = cfg.Tool + "-isolated"
	second, err := newBroker(otherCfg)
	if err != nil {
		t.Fatal(err)
	}
	if err = second.Start(nil); err != nil {
		t.Fatal(err)
	}
	defer second.Stop(nil)
	if _, err = Handle(t.Context(), otherCfg, []string{"service", "stop"}); err != nil {
		t.Fatal(err)
	}
	_, used, err := exchange(t.Context(), cfg, request{Operation: "status"})
	if !used || err != nil {
		t.Fatalf("default instance was stopped: %v %v", used, err)
	}
}

func waitFor(t *testing.T, condition func() bool, description string) {
	t.Helper()
	deadline := time.Now().Add(5 * time.Second)
	for !condition() {
		if time.Now().After(deadline) {
			t.Fatalf("timed out waiting for %s", description)
		}
		time.Sleep(10 * time.Millisecond)
	}
}
