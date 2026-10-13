//go:build integration

package service

import (
	"context"
	"encoding/json"
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"
)

func TestMain(m *testing.M) {
	if os.Getenv("TERMAN_SERVICE_TEST_CHILD") != "1" || len(os.Args) < 3 || os.Args[1] != "--service-child" {
		os.Exit(m.Run())
	}
	for i, arg := range os.Args {
		if arg == "--service-child" && len(os.Args) > i+1 {
			if err := os.WriteFile(os.Args[i+1], []byte(strings.Join(os.Args[i+2:], "|")), 0600); err != nil {
				os.Exit(2)
			}
			if len(os.Args) > i+2 && os.Args[i+2] == "exit" {
				os.Exit(0)
			}
			for {
				if _, err := os.Stat(os.Args[i+1] + ".stop"); err == nil {
					os.Exit(0)
				}
				time.Sleep(10 * time.Millisecond)
			}
		}
	}
	os.Exit(3)
}

func TestBrokerIPCAndLifecycle(t *testing.T) {
	t.Setenv("TERMAN_SERVICE_TEST_CHILD", "1")
	marker := filepath.Join(t.TempDir(), "child.txt")
	cfg := Config{Tool: "service-test-" + filepath.Base(t.TempDir()), ServerArgument: "--service-child"}
	shutdown := make(chan Launch, 2)
	cfg.Shutdown = func(ctx context.Context, launch Launch) error {
		shutdown <- launch
		return errors.New("test forces kill fallback")
	}
	b, err := newBroker(cfg)
	if err != nil {
		t.Fatal(err)
	}
	if err = b.Start(nil); err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = b.Stop(nil) })
	launch := Launch{Name: "original", Cwd: filepath.Dir(marker), Args: []string{marker, "literal;argument", "$(literal)"}}
	pid, used, err := TryStart(t.Context(), cfg, launch)
	if err != nil || !used || pid <= 0 {
		t.Fatalf("start: %d %v %v", pid, used, err)
	}
	deadline := time.Now().Add(5 * time.Second)
	for {
		data, readErr := os.ReadFile(marker)
		if readErr == nil {
			if string(data) != "literal;argument|$(literal)" {
				t.Fatalf("argument corruption: %q", data)
			}
			break
		}
		if time.Now().After(deadline) {
			t.Fatal("child did not execute fixed current binary")
		}
		time.Sleep(10 * time.Millisecond)
	}
	launch.PID = 123
	_, used, err = TryStart(t.Context(), cfg, launch)
	if !used || err == nil {
		t.Fatalf("injected PID: %v %v", used, err)
	}
	launch.PID = 0
	launch.Name = "other"
	launch.Cwd = filepath.Join(t.TempDir(), "missing")
	_, used, err = TryStart(t.Context(), cfg, launch)
	if !used || err == nil {
		t.Fatalf("invalid cwd: %v %v", used, err)
	}
	handled, err := Handle(t.Context(), cfg, []string{"service", "status"})
	if !handled || err != nil {
		t.Fatalf("status: %v %v", handled, err)
	}
	handled, err = Handle(t.Context(), cfg, []string{"service", "stop"})
	if !handled || err != nil {
		t.Fatalf("stop: %v %v", handled, err)
	}
	select {
	case got := <-shutdown:
		if got.PID != pid || got.Name != "original" {
			t.Fatalf("shutdown launch: %+v", got)
		}
	case <-time.After(5 * time.Second):
		t.Fatal("shutdown was not called")
	}
	select {
	case <-b.done:
	case <-time.After(5 * time.Second):
		t.Fatal("broker did not stop and reap children")
	}
	_, used, err = TryStart(t.Context(), cfg, Launch{Name: "unavailable"})
	if used || err != nil {
		t.Fatalf("fallback: %v %v", used, err)
	}
}

func TestBrokerRejectsMalformedIPC(t *testing.T) {
	cfg := Config{Tool: "service-invalid-" + filepath.Base(t.TempDir()), ServerArgument: "--server"}
	b, err := newBroker(cfg)
	if err != nil {
		t.Fatal(err)
	}
	if err = b.Start(nil); err != nil {
		t.Fatal(err)
	}
	defer b.Stop(nil)
	endpoint, err := Endpoint(cfg)
	if err != nil {
		t.Fatal(err)
	}
	conn, err := dialEndpoint(t.Context(), endpoint)
	if err != nil {
		t.Fatal(err)
	}
	defer conn.Close()
	conn.SetDeadline(time.Now().Add(5 * time.Second))
	conn.Write([]byte("{bad json}\n"))
	var reply response
	if err = json.NewDecoder(conn).Decode(&reply); err != nil {
		t.Fatal(err)
	}
	if reply.Error == "" {
		t.Fatal("malformed request accepted")
	}
	second, err := newBroker(cfg)
	if err != nil {
		t.Fatal(err)
	}
	if err = second.Start(nil); err == nil {
		second.Stop(nil)
		t.Fatal("second listener took over endpoint")
	}
}

func TestStatusJSON(t *testing.T) {
	cfg := Config{Tool: "status-test-" + filepath.Base(t.TempDir()), ServerArgument: "--server"}
	b, err := newBroker(cfg)
	if err != nil {
		t.Fatal(err)
	}
	if err = b.Start(nil); err != nil {
		t.Fatal(err)
	}
	defer b.Stop(nil)
	capture, err := os.CreateTemp(t.TempDir(), "status")
	if err != nil {
		t.Fatal(err)
	}
	defer capture.Close()
	original := os.Stdout
	os.Stdout = capture
	handled, handleErr := Handle(t.Context(), cfg, []string{"service", "status", "--json"})
	os.Stdout = original
	if !handled || handleErr != nil {
		t.Fatalf("JSON status: %v %v", handled, handleErr)
	}
	if _, err = capture.Seek(0, 0); err != nil {
		t.Fatal(err)
	}
	var status struct {
		Name     string
		Status   string
		PID      int
		Sessions int
	}
	if err = json.NewDecoder(capture).Decode(&status); err != nil {
		t.Fatal(err)
	}
	if status.Name != cfg.Tool || status.Status != "running" || status.PID != os.Getpid() || status.Sessions != 0 {
		t.Fatalf("status: %+v", status)
	}
}
