package session

import (
	"fmt"
	"strings"
)

func (r *Reactor) activeWindow(window *int) (*Window, error) {
	index := r.state.ActiveWindow
	if window != nil {
		index = *window
	}
	value := r.state.Windows[index]
	if value == nil {
		return nil, fmt.Errorf("window %d not found", index)
	}
	return value, nil
}

func activePane(window *Window, pane *int) (*Pane, error) {
	index := window.ActivePane
	if pane != nil {
		index = *pane
	}
	value := window.Panes[index]
	if value == nil {
		return nil, fmt.Errorf("pane %d not found", index)
	}
	return value, nil
}

func (r *Reactor) input(windowID, paneID *int, data []byte) error {
	window, err := r.activeWindow(windowID)
	if err != nil {
		return err
	}
	if window.Synchronize {
		for _, index := range window.PaneOrder {
			if pane := window.Panes[index]; pane != nil && !pane.Dead {
				if _, err := pane.PTY.Write(data); err != nil {
					return fmt.Errorf("write pane %d: %w", pane.Index, err)
				}
			}
		}
		return nil
	}
	pane, err := activePane(window, paneID)
	if err != nil {
		return err
	}
	if pane.Dead {
		return fmt.Errorf("pane %d has exited", pane.Index)
	}
	if _, err := pane.PTY.Write(data); err != nil {
		return fmt.Errorf("write pane: %w", err)
	}
	return nil
}

func (r *Reactor) splitPane(windowID *int, horizontal bool, command string) error {
	window, err := r.activeWindow(windowID)
	if err != nil {
		return err
	}
	index := window.NextPane
	window.NextPane++
	pane, err := r.startPane(window.Index, index, max(r.state.Cols/2, 1), max((r.state.Rows-1)/2, 1), command)
	if err != nil {
		return err
	}
	if !window.Layout.split(window.ActivePane, index, horizontal) {
		closePane(pane)
		return fmt.Errorf("active pane is missing from layout")
	}
	window.Panes[index] = pane
	window.PaneOrder = append(window.PaneOrder, index)
	window.LastPane, window.ActivePane = window.ActivePane, index
	window.LayoutName = "manual"
	r.resizePanes(window)
	return nil
}

func (r *Reactor) selectWindow(index *int) error {
	if index == nil {
		return fmt.Errorf("window target is required")
	}
	if r.state.Windows[*index] == nil {
		return fmt.Errorf("window %d not found", *index)
	}
	r.state.LastWindow, r.state.ActiveWindow = r.state.ActiveWindow, *index
	return nil
}

func (r *Reactor) selectWindowRelative(direction string) error {
	order := r.state.WindowOrder
	if len(order) == 0 {
		return fmt.Errorf("session has no windows")
	}
	position := 0
	for i, index := range order {
		if index == r.state.ActiveWindow {
			position = i
			break
		}
	}
	if direction == "previous" {
		position = (position - 1 + len(order)) % len(order)
	} else {
		position = (position + 1) % len(order)
	}
	return r.selectWindow(&order[position])
}

func (r *Reactor) selectLastWindow() error {
	if r.state.Windows[r.state.LastWindow] == nil {
		return fmt.Errorf("last window not found")
	}
	return r.selectWindow(&r.state.LastWindow)
}

func (r *Reactor) renameWindow(windowID *int, name string) error {
	window, err := r.activeWindow(windowID)
	if err != nil {
		return err
	}
	if strings.TrimSpace(name) == "" {
		return fmt.Errorf("window name is required")
	}
	window.Name = name
	return nil
}

func (r *Reactor) killWindow(windowID *int) error {
	window, err := r.activeWindow(windowID)
	if err != nil {
		return err
	}
	if len(r.state.Windows) == 1 {
		return fmt.Errorf("cannot remove the last window")
	}
	for _, pane := range window.Panes {
		closePane(pane)
	}
	delete(r.state.Windows, window.Index)
	r.state.WindowOrder = removeInt(r.state.WindowOrder, window.Index)
	if r.state.ActiveWindow == window.Index {
		r.state.ActiveWindow = r.state.WindowOrder[len(r.state.WindowOrder)-1]
	}
	return nil
}

func (r *Reactor) selectPane(windowID, paneID *int, direction string) error {
	window, err := r.activeWindow(windowID)
	if err != nil {
		return err
	}
	target := paneID
	if target == nil && direction != "" {
		value := adjacentPane(window, direction)
		target = &value
	}
	if target == nil {
		return fmt.Errorf("pane target is required")
	}
	if window.Panes[*target] == nil {
		return fmt.Errorf("pane %d not found", *target)
	}
	window.LastPane, window.ActivePane = window.ActivePane, *target
	return nil
}

func adjacentPane(window *Window, direction string) int {
	position := 0
	for i, pane := range window.PaneOrder {
		if pane == window.ActivePane {
			position = i
			break
		}
	}
	if direction == "left" || direction == "up" || direction == "previous" {
		position = (position - 1 + len(window.PaneOrder)) % len(window.PaneOrder)
	} else {
		position = (position + 1) % len(window.PaneOrder)
	}
	return window.PaneOrder[position]
}

func (r *Reactor) swapPane(windowID, sourceID, targetID *int, direction string) error {
	window, err := r.activeWindow(windowID)
	if err != nil {
		return err
	}
	source := window.ActivePane
	if sourceID != nil {
		source = *sourceID
	}
	target := window.LastPane
	if targetID != nil {
		target = *targetID
	} else if direction != "" {
		target = adjacentPane(window, direction)
	}
	if window.Panes[source] == nil || window.Panes[target] == nil {
		return fmt.Errorf("source or target pane not found")
	}
	window.Layout.swap(source, target)
	window.ActivePane = source
	r.resizePanes(window)
	return nil
}

func removeInt(values []int, target int) []int {
	for i, value := range values {
		if value == target {
			return append(values[:i], values[i+1:]...)
		}
	}
	return values
}

func (r *Reactor) killPane(windowID, paneID *int) error {
	window, err := r.activeWindow(windowID)
	if err != nil {
		return err
	}
	pane, err := activePane(window, paneID)
	if err != nil {
		return err
	}
	if len(window.Panes) == 1 {
		return r.killWindow(windowID)
	}
	closePane(pane)
	delete(window.Panes, pane.Index)
	window.PaneOrder = removeInt(window.PaneOrder, pane.Index)
	window.Layout, _ = window.Layout.remove(pane.Index)
	window.ActivePane = window.PaneOrder[0]
	r.resizePanes(window)
	return nil
}

func (r *Reactor) zoomPane(windowID, paneID *int) error {
	window, err := r.activeWindow(windowID)
	if err != nil {
		return err
	}
	if paneID != nil {
		if err := r.selectPane(windowID, paneID, ""); err != nil {
			return err
		}
	}
	window.Zoomed = !window.Zoomed
	r.resizePanes(window)
	return nil
}
