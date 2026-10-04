package app

import (
	"sort"

	"github.com/gdamore/tcell/v3"
)

type menuKind int

const (
	menuSort menuKind = iota
	menuSetup
	menuSignal
	menuUser
)

type menuEntry struct {
	label string
	value string
}

type menuOverlay struct {
	kind     menuKind
	title    string
	entries  []menuEntry
	selected int
	scroll   int
}

func (s *state) openSortMenu() {
	entries := make([]menuEntry, 0, 5)
	for _, key := range []SortKey{SortCPU, SortMemory, SortIO, SortPID, SortName} {
		entries = append(entries, menuEntry{label: key.String(), value: key.String()})
	}
	s.overlay = &menuOverlay{kind: menuSort, title: "Sort by", entries: entries, selected: int(s.sort)}
}

func (s *state) openSetupMenu() {
	s.overlay = &menuOverlay{kind: menuSetup, title: "Setup", entries: s.setupEntries()}
}

func (s *state) setupEntries() []menuEntry {
	return []menuEntry{
		{label: "Tree view: " + onOff(s.tree), value: "tree"},
		{label: "Show command line: " + onOff(s.showCommand), value: "command"},
		{label: "Reverse sort: " + onOff(s.reverse), value: "reverse"},
		{label: "Clear process tags", value: "tags"},
		{label: "Expand all tree nodes", value: "expand"},
	}
}

func (s *state) openSignalMenu() {
	entries := make([]menuEntry, 0, 6)
	for _, signal := range []ProcessSignal{
		SignalTerm, SignalKill, SignalInterrupt, SignalHangup, SignalStop, SignalContinue,
	} {
		entries = append(entries, menuEntry{label: signal.String(), value: signal.String()})
	}
	s.overlay = &menuOverlay{kind: menuSignal, title: "Send signal", entries: entries}
}

func (s *state) openUserMenu() {
	users := make(map[string]bool)
	for _, process := range s.snapshot.Processes {
		if process.User != "" && process.User != "-" {
			users[process.User] = true
		}
	}
	names := make([]string, 0, len(users))
	for user := range users {
		names = append(names, user)
	}
	sort.Strings(names)
	entries := []menuEntry{{label: "All users", value: ""}}
	selected := 0
	for _, user := range names {
		entries = append(entries, menuEntry{label: user, value: user})
		if user == s.userFilter {
			selected = len(entries) - 1
		}
	}
	s.overlay = &menuOverlay{kind: menuUser, title: "Exact user filter", entries: entries, selected: selected}
}

func (s *state) handleOverlay(event *tcell.EventKey) (bool, error) {
	overlay := s.overlay
	switch event.Key() {
	case tcell.KeyEscape:
		s.overlay = nil
	case tcell.KeyUp:
		overlay.selected = clamp(overlay.selected-1, 0, len(overlay.entries)-1)
	case tcell.KeyDown:
		overlay.selected = clamp(overlay.selected+1, 0, len(overlay.entries)-1)
	case tcell.KeyPgUp:
		overlay.selected = clamp(overlay.selected-10, 0, len(overlay.entries)-1)
	case tcell.KeyPgDn:
		overlay.selected = clamp(overlay.selected+10, 0, len(overlay.entries)-1)
	case tcell.KeyHome:
		overlay.selected = 0
	case tcell.KeyEnd:
		overlay.selected = max(0, len(overlay.entries)-1)
	case tcell.KeyEnter:
		s.activateMenuEntry(overlay.selected)
	}
	return false, nil
}

func (s *state) activateMenuEntry(index int) {
	if s.overlay == nil {
		return
	}
	entry, ok := at(s.overlay.entries, index)
	if !ok {
		return
	}
	switch s.overlay.kind {
	case menuSort:
		keys := map[string]SortKey{"CPU": SortCPU, "MEM": SortMemory, "IO": SortIO, "PID": SortPID, "NAME": SortName}
		s.setSort(keys[entry.value])
		s.overlay = nil
	case menuSignal:
		signal := map[string]ProcessSignal{
			"TERM": SignalTerm, "KILL": SignalKill, "INT": SignalInterrupt,
			"HUP": SignalHangup, "STOP": SignalStop, "CONT": SignalContinue,
		}[entry.value]
		s.overlay = nil
		s.confirmSignal(signal)
	case menuUser:
		s.userFilter = entry.value
		s.selected[s.tab], s.scroll[s.tab] = 0, 0
		s.overlay = nil
	case menuSetup:
		s.activateSetup(entry.value)
	}
}

func (s *state) activateSetup(value string) {
	switch value {
	case "tree":
		s.toggleTree()
	case "command":
		s.showCommand = !s.showCommand
	case "reverse":
		s.reverse = !s.reverse
	case "tags":
		clear(s.tags)
	case "expand":
		clear(s.collapsed)
	}
	s.overlay.entries = s.setupEntries()
}
