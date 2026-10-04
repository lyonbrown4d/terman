package session

import (
	"fmt"

	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
)

func (r *Reactor) resizePane(req protocol.Request) error {
	window, err := r.activeWindow(req.Window)
	if err != nil {
		return err
	}
	pane, err := activePane(window, req.Pane)
	if err != nil {
		return err
	}
	delta := req.Delta
	if delta <= 0 {
		delta = 1
	}
	if req.Direction != "" {
		if !window.Layout.resize(pane.Index, req.Direction, float64(delta)*0.03) {
			return fmt.Errorf("no layout edge in direction %s", req.Direction)
		}
	}
	if req.Width > 0 || req.Height > 0 {
		rects := r.layoutRects(window)
		current := rects[pane.Index]
		if req.Width > 0 {
			direction := "right"
			if req.Width < current.W {
				direction = "left"
			}
			_ = window.Layout.resize(pane.Index, direction, float64(abs(req.Width-current.W))*0.03)
		}
		if req.Height > 0 {
			direction := "down"
			if req.Height < current.H {
				direction = "up"
			}
			_ = window.Layout.resize(pane.Index, direction, float64(abs(req.Height-current.H))*0.03)
		}
	}
	r.resizePanes(window)
	return nil
}

func (r *Reactor) selectLayout(windowID *int, name string) error {
	window, err := r.activeWindow(windowID)
	if err != nil {
		return err
	}
	switch name {
	case "", "next":
		names := []string{"even-horizontal", "even-vertical", "tiled"}
		next := 0
		for i, value := range names {
			if value == window.LayoutName {
				next = (i + 1) % len(names)
			}
		}
		name = names[next]
	case "even-horizontal", "even-vertical", "tiled":
	default:
		return fmt.Errorf("unsupported layout %q", name)
	}
	window.Layout = balanced(window.PaneOrder, name)
	window.LayoutName = name
	r.resizePanes(window)
	return nil
}

func (r *Reactor) setSynchronize(windowID *int, enabled *bool) error {
	window, err := r.activeWindow(windowID)
	if err != nil {
		return err
	}
	if enabled == nil {
		window.Synchronize = !window.Synchronize
	} else {
		window.Synchronize = *enabled
	}
	return nil
}

func (r *Reactor) resize(cols, rows int) error {
	if cols < 2 || rows < 2 {
		return fmt.Errorf("terminal size must be at least 2x2")
	}
	r.state.Cols, r.state.Rows = cols, rows
	for _, window := range r.state.Windows {
		r.resizePanes(window)
	}
	return nil
}

func (r *Reactor) layoutRects(window *Window) map[int]protocol.Rect {
	rects := make(map[int]protocol.Rect)
	if window.Zoomed {
		rects[window.ActivePane] = protocol.Rect{W: r.state.Cols, H: max(r.state.Rows-1, 1)}
		return rects
	}
	window.Layout.rects(protocol.Rect{W: r.state.Cols, H: max(r.state.Rows-1, 1)}, rects)
	return rects
}

func (r *Reactor) resizePanes(window *Window) {
	for index, rect := range r.layoutRects(window) {
		pane := window.Panes[index]
		if pane == nil {
			continue
		}
		pane.Width, pane.Height = max(rect.W, 1), max(rect.H, 1)
		_ = pane.PTY.Resize(pane.Width, pane.Height)
		pane.Term.Resize(pane.Width, pane.Height)
	}
}

func abs(value int) int {
	if value < 0 {
		return -value
	}
	return value
}
