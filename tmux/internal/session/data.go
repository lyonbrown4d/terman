package session

import (
	"fmt"
	"sort"
	"strings"
	"time"

	"github.com/charmbracelet/ultraviolet"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
)

func (r *Reactor) capture(windowID, paneID *int) (string, error) {
	window, err := r.activeWindow(windowID)
	if err != nil {
		return "", err
	}
	pane, err := activePane(window, paneID)
	if err != nil {
		return "", err
	}
	return terminalText(pane), nil
}

func terminalText(pane *Pane) string {
	height := pane.Term.ScrollbackLen() + pane.Term.Height()
	lines := make([]string, 0, height)
	for y := range pane.Term.ScrollbackLen() {
		lines = append(lines, terminalLine(
			pane.Term.Width(),
			func(x int) *uv.Cell { return pane.Term.ScrollbackCellAt(x, y) },
		))
	}
	for y := range pane.Term.Height() {
		lines = append(lines, terminalLine(
			pane.Term.Width(),
			func(x int) *uv.Cell { return pane.Term.CellAt(x, y) },
		))
	}
	return strings.TrimRight(strings.Join(lines, "\n"), "\n")
}

func terminalLine(width int, cellAt func(int) *uv.Cell) string {
	var line strings.Builder
	for x := 0; x < width; x++ {
		cell := cellAt(x)
		if cell == nil {
			line.WriteByte(' ')
			continue
		}
		if cell.Width == 0 {
			continue
		}
		if cell.Content == "" {
			line.WriteByte(' ')
			continue
		}
		line.WriteString(cell.Content)
	}
	return strings.TrimRight(line.String(), " ")
}

func (r *Reactor) clearHistory(windowID, paneID *int) error {
	window, err := r.activeWindow(windowID)
	if err != nil {
		return err
	}
	pane, err := activePane(window, paneID)
	if err != nil {
		return err
	}
	pane.History = nil
	pane.Term.ClearScrollback()
	return nil
}

func (r *Reactor) setBuffer(name string, data []byte) (string, error) {
	if name == "" {
		name = fmt.Sprintf("buffer%d", r.state.NextBuffer)
		r.state.NextBuffer++
	}
	copied := append([]byte(nil), data...)
	if _, exists := r.state.Buffers[name]; !exists {
		r.state.BufferOrder = append(r.state.BufferOrder, name)
	}
	r.state.Buffers[name] = Buffer{
		Name: name, Data: copied, Created: time.Now(),
	}
	return name, nil
}

func (r *Reactor) getBuffer(name string) (string, error) {
	if name == "" {
		if len(r.state.BufferOrder) == 0 {
			return "", fmt.Errorf("no buffers")
		}
		name = r.state.BufferOrder[len(r.state.BufferOrder)-1]
	}
	buffer, ok := r.state.Buffers[name]
	if !ok {
		return "", fmt.Errorf("buffer %q not found", name)
	}
	return string(buffer.Data), nil
}

func (r *Reactor) deleteBuffer(name string) error {
	if name == "" {
		if len(r.state.BufferOrder) == 0 {
			return fmt.Errorf("no buffers")
		}
		name = r.state.BufferOrder[len(r.state.BufferOrder)-1]
	}
	if _, ok := r.state.Buffers[name]; !ok {
		return fmt.Errorf("buffer %q not found", name)
	}
	delete(r.state.Buffers, name)
	for i, value := range r.state.BufferOrder {
		if value == name {
			r.state.BufferOrder = append(
				r.state.BufferOrder[:i],
				r.state.BufferOrder[i+1:]...,
			)
			break
		}
	}
	return nil
}

func (r *Reactor) bufferInfos() []protocol.BufferInfo {
	result := make([]protocol.BufferInfo, 0, len(r.state.Buffers))
	for _, name := range r.state.BufferOrder {
		value := r.state.Buffers[name]
		preview := strings.Map(func(char rune) rune {
			if char < 0x20 {
				return ' '
			}
			return char
		}, string(value.Data))
		if len(preview) > 48 {
			preview = preview[:48]
		}
		result = append(result, protocol.BufferInfo{
			Name: name, Bytes: len(value.Data),
			Preview: preview, Created: value.Created,
		})
	}
	return result
}

func (r *Reactor) sessionInfo() protocol.SessionInfo {
	sync := false
	if window := r.state.Windows[r.state.ActiveWindow]; window != nil {
		sync = window.Synchronize
	}
	return protocol.SessionInfo{
		Name:          r.state.Name,
		CreatedAt:     r.state.CreatedAt,
		Windows:       len(r.state.Windows),
		ActiveWindow:  r.state.ActiveWindow,
		Attached:      len(r.state.Clients),
		NextWindow:    r.state.NextWindow,
		Synchronize:   sync,
		SchemaVersion: protocol.SchemaVersion,
	}
}

func (r *Reactor) windowInfos() []protocol.WindowInfo {
	indexes := sortedKeys(r.state.Windows)
	result := make([]protocol.WindowInfo, 0, len(indexes))
	for _, index := range indexes {
		window := r.state.Windows[index]
		result = append(result, protocol.WindowInfo{
			Index:       index,
			Name:        window.Name,
			Active:      index == r.state.ActiveWindow,
			PaneCount:   len(window.Panes),
			ActivePane:  window.ActivePane,
			NextPane:    window.NextPane,
			Layout:      window.LayoutName,
			Synchronize: window.Synchronize,
			Zoomed:      window.Zoomed,
		})
	}
	return result
}

func (r *Reactor) paneInfos(windowID *int) ([]protocol.PaneInfo, error) {
	window, err := r.activeWindow(windowID)
	if err != nil {
		return nil, err
	}
	result := make([]protocol.PaneInfo, 0, len(window.Panes))
	for _, index := range sortedKeys(window.Panes) {
		pane := window.Panes[index]
		result = append(result, protocol.PaneInfo{
			Index: index, Window: window.Index,
			Active: index == window.ActivePane,
			Dead:   pane.Dead, Width: pane.Width, Height: pane.Height,
		})
	}
	return result, nil
}

func (r *Reactor) clientInfos() []protocol.ClientInfo {
	result := make([]protocol.ClientInfo, 0, len(r.state.Clients))
	for _, client := range r.state.Clients {
		result = append(result, protocol.ClientInfo{
			ID: client.ID, Attached: client.Attached,
			Width: client.Width, Height: client.Height,
		})
	}
	sort.Slice(result, func(i, j int) bool {
		return result[i].ID < result[j].ID
	})
	return result
}
