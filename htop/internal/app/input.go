package app

import (
	"context"
	"fmt"
	"time"

	"github.com/gdamore/tcell/v2"
)

type state struct {
	snapshot    Snapshot
	tab         Tab
	sort        SortKey
	reverse     bool
	filter      string
	filterEdit  bool
	selected    [tabCount]int
	scroll      [tabCount]int
	detail      bool
	environment []string
	status      string
	statusAt    time.Time
	confirm     string
}

func newState(cfg Config) *state {
	return &state{sort: cfg.Sort, reverse: cfg.Reverse, filter: cfg.Filter}
}

func (s *state) handleEvent(ctx context.Context, screen tcell.Screen, event tcell.Event) (bool, error) {
	switch event := event.(type) {
	case *tcell.EventResize:
		screen.Sync()
	case *tcell.EventMouse:
		return s.handleMouse(event), nil
	case *tcell.EventKey:
		return s.handleKey(ctx, event)
	}
	return false, nil
}

func (s *state) handleKey(ctx context.Context, event *tcell.EventKey) (bool, error) {
	if s.filterEdit {
		return false, s.editFilter(event)
	}
	if s.confirm != "" {
		if event.Rune() == 'y' || event.Rune() == 'Y' {
			force := s.confirm == "kill"
			s.confirm = ""
			process, ok := s.currentProcess()
			if !ok {
				return false, fmt.Errorf("no process selected")
			}
			if err := signalProcess(process.PID, force); err != nil {
				return false, fmt.Errorf("signal PID %d: %w", process.PID, err)
			}
			s.setStatus(fmt.Sprintf("signal sent to PID %d", process.PID))
			return false, nil
		}
		if event.Key() == tcell.KeyEscape || event.Rune() == 'n' {
			s.confirm = ""
			s.setStatus("signal cancelled")
		}
		return false, nil
	}
	if event.Key() == tcell.KeyCtrlC || event.Key() == tcell.KeyF10 {
		return true, nil
	}
	switch event.Key() {
	case tcell.KeyLeft:
		s.changeTab(-1)
	case tcell.KeyRight, tcell.KeyTAB:
		s.changeTab(1)
	case tcell.KeyUp:
		s.move(-1)
	case tcell.KeyDown:
		s.move(1)
	case tcell.KeyPgUp:
		s.move(-10)
	case tcell.KeyPgDn:
		s.move(10)
	case tcell.KeyHome:
		s.selected[s.tab] = 0
	case tcell.KeyEnd:
		s.selected[s.tab] = max(0, s.currentLength()-1)
	case tcell.KeyEnter:
		s.detail = !s.detail
	case tcell.KeyEscape:
		s.filter = ""
		s.environment = nil
		s.detail = false
	default:
		return s.handleRune(ctx, event.Rune())
	}
	return false, nil
}

func (s *state) handleRune(ctx context.Context, key rune) (bool, error) {
	switch key {
	case 'q':
		return true, nil
	case '1', '2', '3', '4':
		s.tab = Tab(key - '1')
	case 'j':
		s.move(1)
	case 'k':
		s.confirmSignal("terminate")
	case 'K':
		s.confirmSignal("kill")
	case '/':
		s.filterEdit = true
		s.setStatus("filter: type text, Enter applies, Esc cancels")
	case 'r':
		s.reverse = !s.reverse
	case 'c':
		s.setSort(SortCPU)
	case 'm':
		s.setSort(SortMemory)
	case 'i':
		s.setSort(SortIO)
	case 'p':
		s.setSort(SortPID)
	case 'n':
		s.setSort(SortName)
	case 'd':
		s.detail = !s.detail
	case 'e':
		process, ok := s.currentProcess()
		if !ok {
			return false, fmt.Errorf("no process selected")
		}
		environment, err := loadEnvironment(ctx, process.PID)
		if err != nil {
			return false, fmt.Errorf("read PID %d environment: %w", process.PID, err)
		}
		s.environment, s.detail = environment, true
	case '[':
		return false, s.changePriority(-1)
	case ']':
		return false, s.changePriority(1)
	}
	return false, nil
}

func (s *state) editFilter(event *tcell.EventKey) error {
	switch event.Key() {
	case tcell.KeyEnter:
		s.filterEdit = false
		s.selected[s.tab], s.scroll[s.tab] = 0, 0
		s.setStatus("filter applied")
	case tcell.KeyEscape:
		s.filterEdit = false
	case tcell.KeyBackspace, tcell.KeyBackspace2:
		runes := []rune(s.filter)
		if len(runes) != 0 {
			s.filter = string(runes[:len(runes)-1])
		}
	case tcell.KeyRune:
		if event.Modifiers()&tcell.ModCtrl == 0 {
			s.filter += string(event.Rune())
		}
	}
	return nil
}

func (s *state) handleMouse(event *tcell.EventMouse) bool {
	x, y := event.Position()
	buttons := event.Buttons()
	if buttons&tcell.WheelUp != 0 {
		s.move(-3)
		return false
	}
	if buttons&tcell.WheelDown != 0 {
		s.move(3)
		return false
	}
	if buttons&tcell.Button1 == 0 {
		return false
	}
	if y == 0 {
		if tab, ok := tabAt(x); ok {
			s.tab = tab
		}
		return false
	}
	start := s.bodyStart()
	if y > start {
		index := s.scroll[s.tab] + y - start - 1
		if index >= 0 && index < s.currentLength() {
			s.selected[s.tab] = index
		}
	}
	return false
}

func (s *state) changeTab(delta int) {
	next := (int(s.tab) + delta + int(tabCount)) % int(tabCount)
	s.tab = Tab(next)
}

func (s *state) move(delta int) {
	s.selected[s.tab] = clamp(s.selected[s.tab]+delta, 0, max(0, s.currentLength()-1))
}

func (s *state) setSort(key SortKey) {
	if s.sort == key {
		s.reverse = !s.reverse
	} else {
		s.sort, s.reverse = key, false
	}
}

func (s *state) confirmSignal(kind string) {
	if _, ok := s.currentProcess(); !ok {
		s.setStatus("no process selected")
		return
	}
	s.confirm = kind
	s.setStatus("press y to confirm, n or Esc to cancel")
}

func (s *state) changePriority(delta int32) error {
	process, ok := s.currentProcess()
	if !ok {
		return fmt.Errorf("no process selected")
	}
	if err := setProcessPriority(process.PID, process.Nice+delta); err != nil {
		return fmt.Errorf("set PID %d priority: %w", process.PID, err)
	}
	s.setStatus(fmt.Sprintf("PID %d priority changed", process.PID))
	return nil
}

func (s *state) currentProcess() (Process, bool) {
	rows := processRows(s.snapshot, s.sort, s.reverse, s.filter)
	if s.tab == TabOverview || s.tab == TabProcesses {
		return at(rows, s.selected[s.tab])
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
	for _, process := range s.snapshot.Processes {
		if process.PID == pid {
			return process, true
		}
	}
	return Process{}, false
}
