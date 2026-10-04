package app

import (
	"context"
	"fmt"
	"sort"
)

func (s *state) setSort(key SortKey) {
	if s.sort == key {
		s.reverse = !s.reverse
	} else {
		s.sort, s.reverse = key, false
	}
	s.selected[s.tab], s.scroll[s.tab] = 0, 0
}

func (s *state) confirmSignal(signal ProcessSignal) {
	pids := s.targetPIDs()
	if len(pids) == 0 {
		s.setStatus("no process selected")
		return
	}
	s.confirm = &signalConfirmation{signal: signal, pids: pids}
	s.setStatus("press y to confirm, n or Esc to cancel")
}

func (s *state) changePriority(delta int32) error {
	pids := s.targetPIDs()
	if len(pids) == 0 {
		return fmt.Errorf("no process selected")
	}
	for _, pid := range pids {
		process, ok := s.processByPID(pid)
		if !ok {
			continue
		}
		if err := setProcessPriority(pid, process.Nice+delta); err != nil {
			return fmt.Errorf("set PID %d priority: %w", pid, err)
		}
	}
	s.setStatus(fmt.Sprintf("priority changed for %d process(es)", len(pids)))
	return nil
}

func (s *state) toggleFollow() {
	process, ok := s.currentProcess()
	if !ok {
		s.setStatus("no process selected")
		return
	}
	if s.followPID == process.PID {
		s.followPID = 0
		s.setStatus("PID follow disabled")
		return
	}
	s.followPID = process.PID
	s.setStatus(fmt.Sprintf("following PID %d", process.PID))
}

func (s *state) toggleTag() {
	process, ok := s.currentProcess()
	if !ok {
		s.setStatus("no process selected")
		return
	}
	if s.tags[process.PID] {
		delete(s.tags, process.PID)
		s.setStatus(fmt.Sprintf("PID %d untagged", process.PID))
		return
	}
	s.tags[process.PID] = true
	s.setStatus(fmt.Sprintf("PID %d tagged", process.PID))
}

func (s *state) toggleTree() {
	s.tree = !s.tree
	s.selected[s.tab], s.scroll[s.tab] = 0, 0
	s.setStatus(fmt.Sprintf("tree view %s", onOff(s.tree)))
}

func (s *state) setNodeCollapsed(value bool) {
	row, ok := s.selectedProcessRow()
	if !s.tree || !ok || !row.HasChildren {
		s.setStatus("selected process has no visible children")
		return
	}
	s.collapsed[row.PID] = value
	if !value {
		delete(s.collapsed, row.PID)
	}
	s.setStatus(fmt.Sprintf("PID %d children %s", row.PID, map[bool]string{true: "collapsed", false: "expanded"}[value]))
}

func (s *state) toggleCollapseAll() {
	if !s.tree {
		s.tree = true
	}
	if len(s.collapsed) != 0 {
		clear(s.collapsed)
		s.setStatus("all process nodes expanded")
		return
	}
	parents := make(map[int32]bool)
	visible := filteredProcesses(s.snapshot.Processes, s.sort, s.reverse, s.filter, s.userFilter)
	present := make(map[int32]bool, len(visible))
	for _, process := range visible {
		present[process.PID] = true
	}
	for _, process := range visible {
		if present[process.PPID] && process.PPID != process.PID {
			parents[process.PPID] = true
		}
	}
	for pid := range parents {
		s.collapsed[pid] = true
	}
	s.selected[s.tab], s.scroll[s.tab] = 0, 0
	s.setStatus("all process nodes collapsed")
}

func (s *state) openDetail() error {
	process, ok := s.currentProcess()
	if !ok {
		return fmt.Errorf("no process selected")
	}
	s.view, s.viewPID, s.viewOffset = viewDetail, process.PID, 0
	return nil
}

func (s *state) openEnvironment(ctx context.Context) error {
	process, ok := s.currentProcess()
	if !ok {
		return fmt.Errorf("no process selected")
	}
	environment, err := loadEnvironment(ctx, process.PID)
	if err != nil {
		return fmt.Errorf("read PID %d environment: %w", process.PID, err)
	}
	sort.Strings(environment)
	s.environment = environment
	s.view, s.viewPID, s.viewOffset = viewEnvironment, process.PID, 0
	return nil
}

func onOff(value bool) string {
	if value {
		return "on"
	}
	return "off"
}
