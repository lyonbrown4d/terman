package server

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/lyonbrown4d/terman/tmux/internal/store"
)

func TestShutdownRemovesRenamedProcessRecord(t *testing.T) {
	home := t.TempDir()
	t.Setenv("LOCALAPPDATA", home)
	t.Setenv("XDG_CACHE_HOME", home)
	t.Setenv("HOME", home)
	if err := store.Save(store.Record{Name: "original", PID: os.Getpid(), Endpoint: "renamed-endpoint"}); err != nil {
		t.Fatal(err)
	}
	if err := store.Rename("original", "renamed"); err != nil {
		t.Fatal(err)
	}
	if err := store.Save(store.Record{Name: "unrelated", PID: os.Getpid() + 1, Endpoint: "unrelated-endpoint"}); err != nil {
		t.Fatal(err)
	}
	if err := (fileStore{}).Remove("original"); err != nil {
		t.Fatal(err)
	}
	records, err := store.Load()
	if err != nil || len(records) != 1 || records[0].Name != "unrelated" {
		t.Fatalf("records after renamed session stop = %#v, error=%v", records, err)
	}
	entries, err := filepath.Glob(filepath.Join(home, "terman", "tmux", "sessions", "*.json"))
	if err != nil || len(entries) != 1 {
		t.Fatalf("record files after stop = %v, error=%v", entries, err)
	}
}
