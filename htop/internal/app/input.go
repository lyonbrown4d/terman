package app

import (
	"context"
	"fmt"
	"strings"

	"github.com/gdamore/tcell/v3"
)

func (s *state) handleEvent(ctx context.Context, screen tcell.Screen, event tcell.Event) (bool, error) {
	switch event := event.(type) {
	case *tcell.EventResize:
		screen.Sync()
	case *tcell.EventMouse:
		return s.handleMouse(ctx, event)
	case *tcell.EventKey:
		return s.handleKey(ctx, event)
	}
	return false, nil
}

func (s *state) handleKey(ctx context.Context, event *tcell.EventKey) (bool, error) {
	if event.Key() == tcell.KeyCtrlC || event.Key() == tcell.KeyF10 {
		return true, nil
	}
	if s.input != inputNone {
		return false, s.editInput(event)
	}
	if s.confirm != nil {
		return false, s.handleConfirmation(event)
	}
	if s.overlay != nil {
		return s.handleOverlay(event)
	}
	if s.view != viewNone {
		return false, s.handleViewKey(event)
	}
	if handled, quit, err := s.handleFunctionKey(ctx, event.Key()); handled {
		return quit, err
	}
	if event.Key() == tcell.KeyTab && event.Modifiers()&tcell.ModShift != 0 {
		s.changeTab(-1)
		return false, nil
	}
	switch event.Key() {
	case tcell.KeyLeft, tcell.KeyBacktab:
		s.changeTab(-1)
	case tcell.KeyRight, tcell.KeyTab:
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
		return false, s.openDetail()
	case tcell.KeyEscape:
		s.filter, s.userFilter, s.search = "", "", ""
		s.setStatus("filters cleared")
	default:
		return s.handleRune(ctx, event.Str())
	}
	return false, nil
}

func (s *state) handleFunctionKey(ctx context.Context, key tcell.Key) (bool, bool, error) {
	switch key {
	case tcell.KeyF2:
		s.openSetupMenu()
	case tcell.KeyF3:
		s.startInput(inputSearch)
	case tcell.KeyF4:
		s.startInput(inputFilter)
	case tcell.KeyF5:
		s.toggleTree()
	case tcell.KeyF6:
		s.openSortMenu()
	case tcell.KeyF7:
		return true, false, s.changePriority(-1)
	case tcell.KeyF8:
		return true, false, s.changePriority(1)
	case tcell.KeyF9:
		s.openSignalMenu()
	default:
		return false, false, nil
	}
	return true, false, nil
}

func (s *state) handleRune(ctx context.Context, key string) (bool, error) {
	switch key {
	case "q":
		return true, nil
	case "1", "2", "3", "4":
		s.tab = Tab(key[0] - '1')
	case "j":
		s.move(1)
	case "k":
		s.confirmSignal(SignalTerm)
	case "K":
		s.confirmSignal(SignalKill)
	case "/", "s":
		s.startInput(inputSearch)
	case "\\":
		s.startInput(inputFilter)
	case "n":
		if s.search != "" {
			s.selectSearch(true)
		} else {
			s.setSort(SortName)
		}
	case "r":
		s.reverse = !s.reverse
	case "c":
		s.setSort(SortCPU)
	case "m":
		s.setSort(SortMemory)
	case "i":
		s.setSort(SortIO)
	case "p":
		s.setSort(SortPID)
	case "d":
		return false, s.openDetail()
	case "e":
		return false, s.openEnvironment(ctx)
	case "u":
		s.openUserMenu()
	case "F":
		s.toggleFollow()
	case " ":
		s.toggleTag()
	case "U":
		clear(s.tags)
		s.setStatus("all process tags cleared")
	case "-":
		s.setNodeCollapsed(true)
	case "+":
		s.setNodeCollapsed(false)
	case "*":
		s.toggleCollapseAll()
	}
	return false, nil
}

func (s *state) startInput(mode inputMode) {
	s.input = mode
	if mode == inputFilter {
		s.editor, s.editorOriginal = s.filter, s.filter
		s.setStatus("live filter: type text, Enter keeps, Esc restores")
		return
	}
	s.editor, s.editorOriginal = s.search, s.search
	s.setStatus("search: type text, Enter keeps, Esc restores, n finds next")
}

func (s *state) editInput(event *tcell.EventKey) error {
	switch event.Key() {
	case tcell.KeyEnter:
		s.input = inputNone
		s.setStatus("input applied")
		return nil
	case tcell.KeyEscape:
		if s.input == inputFilter {
			s.filter = s.editorOriginal
		} else {
			s.search = s.editorOriginal
			s.selectSearch(false)
		}
		s.input = inputNone
		s.setStatus("input cancelled")
		return nil
	case tcell.KeyBackspace:
		runes := []rune(s.editor)
		if len(runes) != 0 {
			s.editor = string(runes[:len(runes)-1])
		}
	case tcell.KeyRune:
		if event.Modifiers()&tcell.ModCtrl == 0 {
			s.editor += event.Str()
		}
	default:
		return nil
	}
	if s.input == inputFilter {
		s.filter = s.editor
		s.selected[s.tab], s.scroll[s.tab] = 0, 0
	} else {
		s.search = s.editor
		s.selectSearch(false)
	}
	return nil
}

func (s *state) handleConfirmation(event *tcell.EventKey) error {
	if event.Key() == tcell.KeyEscape || event.Str() == "n" || event.Str() == "N" {
		s.confirm = nil
		s.setStatus("signal cancelled")
		return nil
	}
	if event.Str() != "y" && event.Str() != "Y" {
		return nil
	}
	confirmation := s.confirm
	s.confirm = nil
	failures := make([]string, 0)
	for _, pid := range confirmation.pids {
		if err := signalProcess(pid, confirmation.signal); err != nil {
			failures = append(failures, fmt.Sprintf("PID %d: %v", pid, err))
		}
	}
	if len(failures) != 0 {
		return fmt.Errorf("send %s: %s", confirmation.signal, strings.Join(failures, "; "))
	}
	s.setStatus(fmt.Sprintf("%s sent to %d process(es)", confirmation.signal, len(confirmation.pids)))
	return nil
}

func (s *state) handleViewKey(event *tcell.EventKey) error {
	switch event.Key() {
	case tcell.KeyEscape:
		s.closeView()
	case tcell.KeyUp:
		s.viewOffset--
	case tcell.KeyDown:
		s.viewOffset++
	case tcell.KeyPgUp:
		s.viewOffset -= 10
	case tcell.KeyPgDn:
		s.viewOffset += 10
	case tcell.KeyHome:
		s.viewOffset = 0
	case tcell.KeyEnd:
		s.viewOffset = 1 << 30
	}
	if event.Str() == "q" || event.Str() == "d" || event.Str() == "e" {
		s.closeView()
	}
	s.viewOffset = max(0, s.viewOffset)
	return nil
}

func (s *state) closeView() {
	s.view, s.viewPID, s.viewOffset = viewNone, 0, 0
	s.environment = []string{}
}
