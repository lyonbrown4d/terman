package app

import (
	"context"
	"strconv"
	"strings"

	"github.com/gdamore/tcell/v3"
)

func (s *state) handleMouse(ctx context.Context, event *tcell.EventMouse) (bool, error) {
	x, y := event.Position()
	buttons := event.Buttons()
	if buttons&tcell.WheelUp != 0 {
		s.scrollMouse(-3)
		return false, nil
	}
	if buttons&tcell.WheelDown != 0 {
		s.scrollMouse(3)
		return false, nil
	}
	if s.overlay != nil {
		if buttons&tcell.Button1 != 0 {
			s.clickOverlay(x, y)
		}
		return false, nil
	}
	if s.view != viewNone {
		if buttons&tcell.Button3 != 0 || buttons&tcell.Button1 != 0 && y == 0 {
			s.closeView()
		}
		return false, nil
	}
	if buttons&tcell.Button1 != 0 {
		for _, box := range s.hitboxes {
			if x >= box.x1 && x < box.x2 && y >= box.y1 && y < box.y2 {
				return s.activateHitbox(ctx, box.action)
			}
		}
	}
	start := s.bodyStart()
	if y > start {
		index := s.scroll[s.tab] + y - start - 1
		if index >= 0 && index < s.currentLength() {
			s.selected[s.tab] = index
			if buttons&tcell.Button2 != 0 {
				s.toggleTag()
			}
			if buttons&tcell.Button3 != 0 {
				return false, s.openDetail()
			}
		}
	}
	return false, nil
}

func (s *state) scrollMouse(delta int) {
	if s.overlay != nil {
		s.overlay.selected = clamp(s.overlay.selected+delta, 0, len(s.overlay.entries)-1)
		return
	}
	if s.view != viewNone {
		s.viewOffset = max(0, s.viewOffset+delta)
		return
	}
	s.move(delta)
}

func (s *state) clickOverlay(x, y int) {
	if s.overlay == nil {
		return
	}
	for _, box := range s.hitboxes {
		if box.action == "overlay" && x >= box.x1 && x < box.x2 && y >= box.y1 && y < box.y2 {
			index := s.overlay.scroll + y - box.y1
			if index >= 0 && index < len(s.overlay.entries) {
				s.overlay.selected = index
				s.activateMenuEntry(index)
			}
			return
		}
	}
}

func (s *state) activateHitbox(ctx context.Context, action string) (bool, error) {
	if strings.HasPrefix(action, "tab:") {
		value, _ := strconv.Atoi(strings.TrimPrefix(action, "tab:"))
		s.tab = Tab(clamp(value, 0, int(tabCount)-1))
		return false, nil
	}
	switch action {
	case "setup":
		s.openSetupMenu()
	case "search":
		s.startInput(inputSearch)
	case "filter":
		s.startInput(inputFilter)
	case "tree":
		s.toggleTree()
	case "sort":
		s.openSortMenu()
	case "nice-":
		return false, s.changePriority(-1)
	case "nice+":
		return false, s.changePriority(1)
	case "signal":
		s.openSignalMenu()
	case "quit":
		return true, nil
	case "sort-pid":
		s.setSort(SortPID)
	case "sort-user", "sort-name":
		s.setSort(SortName)
	case "sort-cpu":
		s.setSort(SortCPU)
	case "sort-memory":
		s.setSort(SortMemory)
	case "sort-io":
		s.setSort(SortIO)
	case "environment":
		return false, s.openEnvironment(ctx)
	}
	return false, nil
}
