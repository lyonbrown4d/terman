package logging

import (
	"context"
	"log/slog"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/arcgolabs/dix"
)

func TestModuleProvidesAndClosesSlogLogger(t *testing.T) {
	directory := t.TempDir()
	bundle, err := New(Config{Component: "test-app", Level: "debug", Directory: directory})
	if err != nil {
		t.Fatalf("create logging module: %v", err)
	}
	runtime, err := dix.New(
		"test-app",
		dix.WithLogger(bundle.Logger),
		dix.Modules(bundle.Module),
	).Start(context.Background())
	if err != nil {
		_ = bundle.Close()
		t.Fatalf("start application: %v", err)
	}
	logger, err := runtime.Container().Resolve[*slog.Logger]()
	if err != nil {
		t.Fatalf("resolve slog logger: %v", err)
	}
	logger.Info("module test event", "scope", "test")
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()
	if err := bundle.Stop(ctx, runtime); err != nil {
		t.Fatalf("stop application: %v", err)
	}
	data, err := os.ReadFile(filepath.Join(directory, "test-app.log"))
	if err != nil {
		t.Fatalf("read log file: %v", err)
	}
	output := string(data)
	for _, expected := range []string{"module test event", "test-app", "scope"} {
		if !strings.Contains(output, expected) {
			t.Errorf("log output does not contain %q: %s", expected, output)
		}
	}
}
