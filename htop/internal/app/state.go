package app

import (
	"fmt"
	"sort"
	"strings"
	"time"

	"github.com/gdamore/tcell/v3"
)

type inputMode int

const (
	inputNone inputMode = iota
	inputSearch
	inputFilter
)

type viewMode int

const (
	viewNone viewMode = iota
	viewDetail
	viewEnvironment
)

type signalConfirmation struct {
	signal ProcessSignal
	pids   []int32
}

type hitbox struct {
	x1, y1 int
	x2, y2 int
	action string
}

type state struct {
	snapshot Snapshot
	tab      Tab
	sort     SortKey
	reverse  bool

	filter         string
	userFilter     string
	search         string
	input          inputMode
	editor         string
	editorOriginal string

	selected  [tabCount]int
	scroll    [tabCount]int
	tree      bool
	collapsed map[int32]bool
	tags      map[int32]bool
	followPID int32

	showCommand bool
	overlay     *menuOverlay
	view        viewMode
	viewPID     int32
	viewOffset  int
	environment []string
	confirm     *signalConfirmation

	status   string
	statusAt time.Time
	hitboxes []hitbox
}

func newState(cfg Config) *state {
	return &state{
		sort: cfg.Sort, reverse: cfg.Reverse, filter: cfg.Filter,
		showCommand: true, collapsed: make(map[int32]bool), tags: make(map[int32]bool),
		hitboxes: make([]hitbox, 0, 16),
	}
}

func (s *state) visibleProcessRows() []ProcessRow {
	processes := filteredProcesses(s.snapshot.Processes, s.sort, s.reverse, s.filter, s.userFilter)
	if s.tree {
		return treeProcessRows(processes, s.collapsed)
	}
	rows := make([]ProcessRow, 0, len(processes))
	for _, process := range processes {
		rows = append(rows, ProcessRow{Process: process})
	}
	return rows
}

func (s *state) currentLength() int {
	switch s.tab {
	case TabOverview, TabProcesses:
		return len(s.visibleProcessRows())
	case TabIO:
		return len(ioRows(s.snapshot, s.reverse))
	default:
		return len(s.snapshot.Connections)
	}
}

func (s *state) currentProcess() (Process, bool) {
	if s.tab == TabOverview || s.tab == TabProcesses {
		row, ok := at(s.visibleProcessRows(), s.selected[s.tab])
		return row.Process, ok
	}
	var pid int32
	if s.tab == TabIO {
		row, ok := at(ioRows(s.snapshot, s.reverse), s.selected[s.tab])
		if !ok {
			return Process{}, false
		}
		pid = row.PID
	} else {
		row, ok := at(s.snapshot.Connections, s.selected[s.tab])
		if !ok {
			return Process{}, false
		}
		pid = row.PID
	}
	return s.processByPID(pid)
}

func (s *state) processByPID(pid int32) (Process, bool) {
	for _, process := range s.snapshot.Processes {
		if process.PID == pid {
			return process, true
		}
	}
	return Process{}, false
}

func (s *state) selectedProcessRow() (ProcessRow, bool) {
	if s.tab != TabOverview && s.tab != TabProcesses {
		return ProcessRow{}, false
	}
	return at(s.visibleProcessRows(), s.selected[s.tab])
}

func (s *state) targetPIDs() []int32 {
	if len(s.tags) != 0 {
		pids := make([]int32, 0, len(s.tags))
		for pid := range s.tags {
			if _, ok := s.processByPID(pid); ok {
				pids = append(pids, pid)
			}
		}
		sort.Slice(pids, func(i, j int) bool { return pids[i] < pids[j] })
		if len(pids) != 0 {
			return pids
		}
	}
	process, ok := s.currentProcess()
	if !ok {
		return []int32{}
	}
	return []int32{process.PID}
}

func (s *state) normalize(screen tcell.Screen) {
	total := s.currentLength()
	if s.followPID != 0 && (s.tab == TabOverview || s.tab == TabProcesses) {
		found := false
		for index, row := range s.visibleProcessRows() {
			if row.PID == s.followPID {
				s.selected[s.tab] = index
				found = true
				break
			}
		}
		if !found {
			s.followPID = 0
			s.setStatus("followed process exited or is hidden")
		}
	}
	s.selected[s.tab] = clamp(s.selected[s.tab], 0, max(0, total-1))
	_, height := screen.Size()
	rows := max(0, height-s.bodyStart()-3)
	s.scroll[s.tab] = clamp(s.scroll[s.tab], 0, max(0, total-rows))
	if s.view != viewNone {
		s.viewOffset = max(0, s.viewOffset)
	}
}

func (s *state) bodyStart() int {
	switch s.tab {
	case TabOverview:
		return 5
	case TabNetwork:
		return 2 + min(len(s.snapshot.Interfaces), 3)
	default:
		return 1
	}
}

func (s *state) setStatus(value string) {
	s.status, s.statusAt = value, time.Now()
}

func (s *state) move(delta int) {
	s.selected[s.tab] = clamp(s.selected[s.tab]+delta, 0, max(0, s.currentLength()-1))
}

func (s *state) changeTab(delta int) {
	s.tab = Tab((int(s.tab) + delta + int(tabCount)) % int(tabCount))
}

func (s *state) selectSearch(next bool) {
	needle := strings.ToLower(strings.TrimSpace(s.search))
	if needle == "" || (s.tab != TabOverview && s.tab != TabProcesses) {
		return
	}
	rows := s.visibleProcessRows()
	start := 0
	if next && len(rows) != 0 {
		start = (s.selected[s.tab] + 1) % len(rows)
	}
	for offset := range len(rows) {
		index := (start + offset) % len(rows)
		row := rows[index]
		haystack := strings.ToLower(fmt.Sprintf("%d %s %s %s", row.PID, row.User, row.Name, row.Command))
		if strings.Contains(haystack, needle) {
			s.selected[s.tab] = index
			return
		}
	}
	s.setStatus("search text not found")
}
