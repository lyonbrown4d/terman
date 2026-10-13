//go:build integration

package e2e

import (
	"context"
	"encoding/json"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/lyonbrown4d/terman/common/modules/service"
	"github.com/lyonbrown4d/terman/screen/internal/client"
	"github.com/lyonbrown4d/terman/screen/internal/store"
	"github.com/lyonbrown4d/terman/screen/internal/transport"
	"github.com/lyonbrown4d/terman/testkit/processtest"
)

func TestServiceManagedSessionLifecycle(t *testing.T) {
	h := newHarness(t)
	help, err := h.run("--help")
	if err != nil || !strings.Contains(help, "terman-screen service install|uninstall|start|stop|restart|status|run") {
		t.Fatalf("service commands missing from help: %v\n%s", err, help)
	}
	t.Setenv("HOME", h.home)
	t.Setenv("USERPROFILE", h.home)
	t.Setenv("LOCALAPPDATA", filepath.Join(h.home, "local"))
	t.Setenv("XDG_CACHE_HOME", filepath.Join(h.home, "cache"))
	t.Setenv("TERMAN_SERVICE_INSTANCE", h.prefix)
	h.env = processtest.Environment(h.env, "TERMAN_SERVICE_INSTANCE="+h.prefix)
	endpoint, err := service.Endpoint(service.Config{Tool: "terman-screen", ServerArgument: "--__screen-server"})
	if err != nil {
		t.Fatal(err)
	}
	ctx, cancel := context.WithCancel(context.Background())
	process, err := processtest.Start(ctx, moduleRoot, screenBinary, h.env, "service", "run")
	if err != nil {
		cancel()
		t.Fatal(err)
	}
	t.Cleanup(func() {
		cleanupCtx, cleanupCancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cleanupCancel()
		_, _ = processtest.Run(cleanupCtx, moduleRoot, screenBinary, h.env, "service", "stop")
		if err := process.Wait(cleanupCtx); err != nil {
			_ = process.Kill()
		}
		cancel()
	})
	readyCtx, readyCancel := context.WithTimeout(t.Context(), 10*time.Second)
	defer readyCancel()
	if !processtest.Wait(readyCtx, 20*time.Millisecond, func() bool {
		dialCtx, dialCancel := context.WithTimeout(readyCtx, 100*time.Millisecond)
		defer dialCancel()
		conn, err := transport.Dial(dialCtx, endpoint)
		if err == nil {
			_ = conn.Close()
		}
		return err == nil
	}) {
		t.Fatalf("service did not become ready\n%s", process.Output())
	}
	initial := h.start("managed")
	h.start("second")
	if initial.PID == process.PID() {
		t.Fatal("session must run in its own child process")
	}
	output, err := h.run("service", "status", "--json")
	if err != nil {
		t.Fatalf("service status: %v\n%s", err, output)
	}
	var status struct {
		Status   string `json:"status"`
		PID      int    `json:"pid"`
		Sessions int    `json:"sessions"`
	}
	if err := json.Unmarshal([]byte(output), &status); err != nil {
		t.Fatal(err)
	}
	if status.Status != "running" || status.PID != process.PID() || status.Sessions != 2 {
		t.Fatalf("broker ownership status = %#v", status)
	}
	h.control("managed", "info")
	h.control("managed", "sessionname", h.session("renamed"))
	h.started[h.session("renamed")] = true
	renamed, found := h.find("renamed")
	if !found || renamed.PID != initial.PID || renamed.Endpoint != initial.Endpoint {
		t.Fatalf("renamed session = %#v, found=%v", renamed, found)
	}
	h.control("renamed", "info")
	output, err = h.run("service", "stop")
	if err != nil {
		t.Fatalf("stop service: %v\n%s", err, output)
	}
	stopCtx, stopCancel := context.WithTimeout(t.Context(), 10*time.Second)
	defer stopCancel()
	if err := process.Wait(stopCtx); err != nil {
		t.Fatalf("service shutdown: %v\n%s", err, process.Output())
	}
	if strings.Contains(process.Output(), "session required forced shutdown") {
		t.Fatalf("service failed graceful shutdown\n%s", process.Output())
	}
	records, err := store.List()
	if err != nil || len(records) != 0 {
		t.Fatalf("records survived service stop: %#v, error=%v", records, err)
	}
	probeCtx, probeCancel := context.WithTimeout(t.Context(), time.Second)
	defer probeCancel()
	if client.Ping(probeCtx, initial.Endpoint) {
		t.Fatal("managed session endpoint survived service stop")
	}
	h.start("fallback")
	h.control("fallback", "info")
	h.control("fallback", "quit")
}
