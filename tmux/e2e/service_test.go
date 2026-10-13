//go:build integration

package e2e

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"
	"time"

	"github.com/lyonbrown4d/terman/common/modules/service"
	"github.com/lyonbrown4d/terman/testkit/processtest"
	"github.com/lyonbrown4d/terman/tmux/internal/ipc"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
	"github.com/lyonbrown4d/terman/tmux/internal/store"
)

func TestServiceManagedSessionLifecycle(t *testing.T) {
	_, source, _, _ := runtime.Caller(0)
	module := filepath.Dir(filepath.Dir(source))
	home := t.TempDir()
	binary := filepath.Join(home, processtest.ExecutableName("terman-tmux"))
	buildCtx, buildCancel := context.WithTimeout(t.Context(), 2*time.Minute)
	defer buildCancel()
	if data, err := processtest.Build(buildCtx, module, binary, "./cmd/terman-tmux", processtest.GlobalGoEnvironment()); err != nil {
		t.Fatalf("build tmux: %v\n%s", err, data)
	}
	t.Setenv("HOME", home)
	t.Setenv("USERPROFILE", home)
	t.Setenv("LOCALAPPDATA", filepath.Join(home, "local"))
	t.Setenv("XDG_CACHE_HOME", filepath.Join(home, "cache"))
	prefix := fmt.Sprintf("tmux-e2e-%d", os.Getpid())
	t.Setenv("TERMAN_SERVICE_INSTANCE", prefix)
	env := os.Environ()
	run := func(args ...string) (string, error) {
		ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
		defer cancel()
		data, err := processtest.Run(ctx, module, binary, env, args...)
		return string(data), err
	}
	mustRun := func(args ...string) string {
		output, err := run(args...)
		if err != nil {
			t.Fatalf("tmux %q: %v\n%s", args, err, output)
		}
		return output
	}
	if help := mustRun("--help"); !strings.Contains(help, "terman-tmux service install|uninstall|start|stop|restart|status|run") {
		t.Fatalf("service commands missing from help:\n%s", help)
	}
	endpoint, err := service.Endpoint(service.Config{Tool: "terman-tmux", ServerArgument: "__server"})
	if err != nil {
		t.Fatal(err)
	}
	ctx, cancel := context.WithCancel(context.Background())
	process, err := processtest.Start(ctx, module, binary, env, "service", "run")
	if err != nil {
		cancel()
		t.Fatal(err)
	}
	t.Cleanup(func() {
		cleanupCtx, cleanupCancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cleanupCancel()
		_, _ = processtest.Run(cleanupCtx, module, binary, env, "service", "stop")
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
		conn, err := ipc.Dial(dialCtx, endpoint)
		if err == nil {
			_ = conn.Close()
		}
		return err == nil
	}) {
		t.Fatalf("service did not become ready\n%s", process.Output())
	}
	managed, second, renamedName := prefix+"-managed", prefix+"-second", prefix+"-renamed"
	mustRun("new-session", "-d", "-s", managed)
	mustRun("new-session", "-d", "-s", second)
	t.Cleanup(func() { _, _ = run("kill-server") })
	var status struct {
		Status   string `json:"status"`
		PID      int    `json:"pid"`
		Sessions int    `json:"sessions"`
	}
	if err := json.Unmarshal([]byte(mustRun("service", "status", "--json")), &status); err != nil {
		t.Fatal(err)
	}
	if status.Status != "running" || status.PID != process.PID() || status.Sessions != 2 {
		t.Fatalf("broker ownership status = %#v", status)
	}
	find := func(name string) protocol.SessionInfo {
		var list struct {
			Sessions []protocol.SessionInfo `json:"sessions"`
		}
		if err := json.Unmarshal([]byte(mustRun("list-sessions", "--json")), &list); err != nil {
			t.Fatal(err)
		}
		for _, item := range list.Sessions {
			if item.Name == name {
				return item
			}
		}
		t.Fatalf("session %q missing from list: %#v", name, list)
		return protocol.SessionInfo{}
	}
	initial := find(managed)
	if initial.ServerPID <= 0 || initial.ServerPID == process.PID() {
		t.Fatalf("session child PID = %d, service PID = %d", initial.ServerPID, process.PID())
	}
	mustRun("list-panes", "-t", managed, "--json")
	mustRun("rename-session", "-t", managed, renamedName)
	renamed := find(renamedName)
	if renamed.ServerPID != initial.ServerPID || renamed.Endpoint != initial.Endpoint {
		t.Fatalf("rename changed process/endpoint: %#v => %#v", initial, renamed)
	}
	mustRun("has-session", "-t", renamedName)
	mustRun("list-panes", "-t", renamedName, "--json")
	mustRun("service", "stop")
	stopCtx, stopCancel := context.WithTimeout(t.Context(), 10*time.Second)
	defer stopCancel()
	if err := process.Wait(stopCtx); err != nil {
		t.Fatalf("service shutdown: %v\n%s", err, process.Output())
	}
	if strings.Contains(process.Output(), "session required forced shutdown") {
		t.Fatalf("service failed graceful shutdown\n%s", process.Output())
	}
	records, err := store.Load()
	if err != nil || len(records) != 0 {
		t.Fatalf("records survived service stop: %#v, error=%v", records, err)
	}
	if ipc.Ping(initial.Endpoint) {
		t.Fatal("managed session endpoint survived service stop")
	}
	mustRun("new-session", "-d", "-s", prefix+"-fallback")
	mustRun("has-session", "-t", prefix+"-fallback")
	mustRun("kill-session", "-t", prefix+"-fallback")
}
