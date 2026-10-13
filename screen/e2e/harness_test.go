//go:build integration

package e2e

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"
	"sync/atomic"
	"testing"
	"time"

	"github.com/lyonbrown4d/terman/testkit/processtest"
	"github.com/lyonbrown4d/terman/testkit/ptytest"
)

const readyText = "TERMAN_SCREEN_E2E_READY"

var (
	assetDirectory string
	screenBinary   string
	sessionBinary  string
	moduleRoot     string
	harnessID      atomic.Uint64
)

type record struct {
	Name     string `json:"name"`
	PID      int    `json:"pid"`
	Endpoint string `json:"endpoint"`
}

type harness struct {
	t       *testing.T
	home    string
	env     []string
	prefix  string
	started map[string]bool
}

func TestMain(m *testing.M) {
	_, source, _, _ := runtime.Caller(0)
	moduleRoot = filepath.Dir(filepath.Dir(source))
	var err error
	assetDirectory, err = os.MkdirTemp("", "terman-screen-e2e-")
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	screenBinary = filepath.Join(
		assetDirectory,
		processtest.ExecutableName("terman-screen"),
	)
	sessionBinary = filepath.Join(
		assetDirectory,
		processtest.ExecutableName("screen-session"),
	)
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Minute)
	defer cancel()
	if output, buildErr := processtest.Build(
		ctx,
		moduleRoot,
		screenBinary,
		"./cmd/terman-screen",
		processtest.GlobalGoEnvironment(),
	); buildErr != nil {
		fmt.Fprintf(os.Stderr, "%v\n%s", buildErr, output)
		os.Exit(1)
	}
	if output, buildErr := processtest.Build(
		ctx,
		moduleRoot,
		sessionBinary,
		"./e2e/testdata/session",
		processtest.GlobalGoEnvironment(),
	); buildErr != nil {
		fmt.Fprintf(os.Stderr, "%v\n%s", buildErr, output)
		os.Exit(1)
	}
	code := m.Run()
	_ = os.RemoveAll(assetDirectory)
	os.Exit(code)
}

func newHarness(t *testing.T) *harness {
	t.Helper()
	home := t.TempDir()
	env := processtest.Environment(
		os.Environ(),
		"HOME="+home,
		"USERPROFILE="+home,
		"XDG_CACHE_HOME="+filepath.Join(home, "cache"),
		"LOCALAPPDATA="+filepath.Join(home, "local"),
		"TERM=xterm-256color",
	)
	prefix := fmt.Sprintf("e2e-%d-%d", os.Getpid(), harnessID.Add(1))
	h := &harness{
		t: t, home: home, env: env, prefix: prefix,
		started: make(map[string]bool),
	}
	t.Cleanup(h.cleanup)
	return h
}

func (h *harness) start(name string) record {
	h.t.Helper()
	actual := h.session(name)
	command := quoteCommand(sessionBinary)
	output, err := h.run("-d", "-S", actual, "--command", command)
	if err != nil {
		h.t.Fatalf("start session: %v\n%s", err, output)
	}
	h.started[actual] = true
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	var current record
	if !processtest.Wait(ctx, 25*time.Millisecond, func() bool {
		item, found := h.find(name)
		current = item
		return found && current.PID > 0
	}) {
		h.t.Fatalf("session %q did not become ready", name)
	}
	return current
}

func (h *harness) run(args ...string) (string, error) {
	h.t.Helper()
	ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
	defer cancel()
	output, err := processtest.Run(ctx, moduleRoot, screenBinary, h.env, args...)
	return string(output), err
}

func (h *harness) attach(name string, multi bool) *ptytest.Client {
	h.t.Helper()
	flag := "-r"
	if multi {
		flag = "-x"
	}
	client, err := ptytest.Start(
		h.t.Context(),
		moduleRoot,
		screenBinary,
		h.env,
		flag,
		h.session(name),
	)
	if err != nil {
		h.t.Fatalf("attach session %q: %v", name, err)
	}
	return client
}

func (h *harness) control(name string, command ...string) string {
	h.t.Helper()
	args := append([]string{"-S", h.session(name), "-X"}, command...)
	output, err := h.run(args...)
	if err != nil {
		h.t.Fatalf("control %q: %v\n%s", command, err, output)
	}
	return output
}

func (h *harness) find(name string) (record, bool) {
	actual := h.session(name)
	output, err := h.run("-ls", "--json")
	if err != nil {
		return record{}, false
	}
	var records []record
	if json.Unmarshal([]byte(output), &records) != nil {
		return record{}, false
	}
	for _, item := range records {
		if item.Name == actual {
			return item, true
		}
	}
	return record{}, false
}

func (h *harness) session(name string) string {
	return h.prefix + "-" + name
}

func (h *harness) detach(client *ptytest.Client) {
	h.t.Helper()
	if err := client.Write([]byte{0x01, 'd'}); err != nil {
		h.t.Fatalf("send detach shortcut: %v", err)
	}
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	if err := client.Wait(ctx); err != nil {
		h.t.Fatalf("wait for detached client: %v", err)
	}
}

func (h *harness) cleanup() {
	for name := range h.started {
		_, _ = h.run("-S", name, "-X", "quit")
	}
	_, _ = h.run("-wipe")
}

func quoteCommand(value string) string {
	if runtime.GOOS == "windows" {
		return value
	}
	return "'" + strings.ReplaceAll(value, "'", "'\\''") + "'"
}

func integerEnvironment(name string, fallback int) int {
	value, err := strconv.Atoi(os.Getenv(name))
	if err != nil || value < 1 {
		return fallback
	}
	return value
}

func durationEnvironment(name string, fallback time.Duration) time.Duration {
	value, err := time.ParseDuration(os.Getenv(name))
	if err != nil || value <= 0 {
		return fallback
	}
	return value
}
