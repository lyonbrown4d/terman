package main

import (
	"context"
	"testing"
	"time"

	"github.com/lyonbrown4d/terman/htop/internal/app"
)

func TestCommandConfigPrecedence(t *testing.T) {
	t.Setenv("TERMAN_HTOP_REFRESH_MS", "2500")
	t.Setenv("TERMAN_HTOP_SORT", "memory")
	t.Setenv("TERMAN_HTOP_FILTER", "worker")
	var received app.Config
	command := newRootCommand(func(_ context.Context, config app.Config) error {
		received = config
		return nil
	})
	command.SetArgs([]string{"--sort", "pid", "--reverse"})

	if err := command.ExecuteContext(context.Background()); err != nil {
		t.Fatalf("execute command: %v", err)
	}
	if received.Refresh != 2500*time.Millisecond {
		t.Errorf("refresh = %s, want 2.5s", received.Refresh)
	}
	if received.Sort != app.SortPID {
		t.Errorf("sort = %s, want PID", received.Sort)
	}
	if received.Filter != "worker" {
		t.Errorf("filter = %q, want worker", received.Filter)
	}
	if !received.Reverse {
		t.Error("reverse flag did not override the default")
	}
}

func TestCommandRejectsInvalidRefresh(t *testing.T) {
	command := newRootCommand(func(context.Context, app.Config) error {
		t.Fatal("runner called for invalid config")
		return nil
	})
	command.SetArgs([]string{"--refresh-ms", "99"})
	if err := command.ExecuteContext(context.Background()); err == nil {
		t.Fatal("expected invalid refresh error")
	}
}

func TestDixCompositionBuildsRunner(t *testing.T) {
	runner, err := composeRunner(app.Config{Refresh: time.Second, Sort: app.SortCPU})
	if err != nil {
		t.Fatalf("compose runner: %v", err)
	}
	if runner == nil {
		t.Fatal("compose runner returned nil")
	}
}
