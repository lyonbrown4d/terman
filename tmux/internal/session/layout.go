package session

import "github.com/lyonbrown4d/terman/tmux/internal/protocol"

type Layout struct {
	Leaf       bool
	Pane       int
	Horizontal bool
	Ratio      float64
	First      *Layout
	Second     *Layout
}

func leaf(pane int) *Layout { return &Layout{Leaf: true, Pane: pane} }

func (l *Layout) split(pane, next int, horizontal bool) bool {
	if l == nil {
		return false
	}
	if l.Leaf {
		if l.Pane != pane {
			return false
		}
		old := l.Pane
		*l = Layout{Horizontal: horizontal, Ratio: 0.5, First: leaf(old), Second: leaf(next)}
		return true
	}
	return l.First.split(pane, next, horizontal) || l.Second.split(pane, next, horizontal)
}

func (l *Layout) contains(pane int) bool {
	if l == nil {
		return false
	}
	if l.Leaf {
		return l.Pane == pane
	}
	return l.First.contains(pane) || l.Second.contains(pane)
}

func (l *Layout) remove(pane int) (*Layout, bool) {
	if l == nil {
		return nil, false
	}
	if l.Leaf {
		if l.Pane == pane {
			return nil, true
		}
		return l, false
	}
	if next, ok := l.First.remove(pane); ok {
		if next == nil {
			return l.Second, true
		}
		l.First = next
		return l, true
	}
	if next, ok := l.Second.remove(pane); ok {
		if next == nil {
			return l.First, true
		}
		l.Second = next
		return l, true
	}
	return l, false
}

func (l *Layout) swap(a, b int) {
	if l == nil {
		return
	}
	if l.Leaf {
		switch l.Pane {
		case a:
			l.Pane = b
		case b:
			l.Pane = a
		}
		return
	}
	l.First.swap(a, b)
	l.Second.swap(a, b)
}

func (l *Layout) rects(rect protocol.Rect, out map[int]protocol.Rect) {
	if l == nil {
		return
	}
	if l.Leaf {
		out[l.Pane] = rect
		return
	}
	ratio := min(max(l.Ratio, 0.15), 0.85)
	a, b := rect, rect
	if l.Horizontal {
		first := max(1, int(float64(rect.W)*ratio))
		if first >= rect.W {
			first = max(1, rect.W-1)
		}
		a.W, b.X, b.W = first, rect.X+first, rect.W-first
	} else {
		first := max(1, int(float64(rect.H)*ratio))
		if first >= rect.H {
			first = max(1, rect.H-1)
		}
		a.H, b.Y, b.H = first, rect.Y+first, rect.H-first
	}
	l.First.rects(a, out)
	l.Second.rects(b, out)
}

func (l *Layout) resize(pane int, direction string, delta float64) bool {
	if l == nil || l.Leaf {
		return false
	}
	inFirst := l.First.contains(pane)
	inSecond := l.Second.contains(pane)
	wantsHorizontal := direction == "left" || direction == "right"
	if l.Horizontal == wantsHorizontal && (inFirst || inSecond) {
		sign := delta
		if direction == "left" {
			if inFirst {
				sign = -delta
			}
		} else if inSecond {
			sign = -delta
		}
		l.Ratio = min(max(l.Ratio+sign, 0.15), 0.85)
		return true
	}
	if inFirst && l.First.resize(pane, direction, delta) {
		return true
	}
	return inSecond && l.Second.resize(pane, direction, delta)
}

func balanced(panes []int, name string) *Layout {
	if len(panes) == 0 {
		return nil
	}
	if len(panes) == 1 {
		return leaf(panes[0])
	}
	mid := len(panes) / 2
	horizontal := name != "even-vertical"
	if name == "tiled" {
		horizontal = len(panes)%2 == 0
	}
	return &Layout{
		Horizontal: horizontal, Ratio: float64(mid) / float64(len(panes)),
		First: balanced(panes[:mid], name), Second: balanced(panes[mid:], name),
	}
}
