//go:build integration

package e2e

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestGNUCompatibilityMatrixIsMaintained(t *testing.T) {
	data, err := os.ReadFile(filepath.Join(moduleRoot, "COMPATIBILITY.md"))
	if err != nil {
		t.Fatalf("read compatibility matrix: %v", err)
	}
	matrix := string(data)
	for _, heading := range []string{
		"## Command-line compatibility",
		"## Runtime command compatibility",
		"## Default shortcut compatibility",
		"## Validation",
	} {
		if !strings.Contains(matrix, heading) {
			t.Errorf("missing matrix section %q", heading)
		}
	}
	for _, token := range []string{
		"`-r name`", "`detach`", "`-x name`", "`screen`", "`select`",
		"`next`", "`prev`", "`last`", "`kill`", "`quit`", "`title`",
		"`sessionname`", "`info`", "`stuff`", "`width`", "`height`",
		"`split`", "`focus`", "`remove`", "`only`", "`resize`",
		"`scrollback`", "`copy`", "`register`", "`readreg`", "`paste`",
		"`hardcopy`", "`log`", "`number`", "`lastmsg`",
	} {
		if !strings.Contains(matrix, token) {
			t.Errorf("token %q is absent from compatibility matrix", token)
		}
	}
}

func TestSupportedGNUCommandsReachRealServer(t *testing.T) {
	h := newHarness(t)
	h.start("compat")
	registerPath := filepath.Join(h.home, "register.txt")
	if err := os.WriteFile(registerPath, []byte("file-register"), 0o600); err != nil {
		t.Fatalf("write register fixture: %v", err)
	}
	commands := [][]string{
		{"info"}, {"dinfo"}, {"version"}, {"number"}, {"lastmsg"},
		{"windows"}, {"windowlist"}, {"title", "compat-window"},
		{"width", "90", "25"}, {"height", "30"}, {"scrollback", "1000"},
		{"defscrollback", "2000"},
		{"split"}, {"focus"}, {"resize", "8"}, {"fit"}, {"redisplay"},
		{"remove"}, {"only"}, {"register", "a", "payload"},
		{"readreg", "a", registerPath}, {"bufferfile", "buffer.txt"},
		{"hardcopydir", h.home},
		{"logfile", "screen.log"}, {"logtstamp", "on"},
	}
	for _, command := range commands {
		t.Run(strings.Join(command, "-"), func(t *testing.T) {
			h.control("compat", command...)
		})
	}
	renamed := h.session("compat-session")
	h.control("compat", "sessionname", renamed)
	delete(h.started, h.session("compat"))
	h.started[renamed] = true
}
