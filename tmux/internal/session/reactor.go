package session

import (
	"fmt"
	"sort"
	"strings"
	"time"

	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
)

func (r *Reactor) loop() {
	ticker := time.NewTicker(time.Second)
	defer ticker.Stop()
	defer close(r.done)
	defer r.cancel()
	for {
		select {
		case <-r.ctx.Done():
			r.closeAll()
			return
		case raw := <-r.events:
			switch event := raw.(type) {
			case requestEvent:
				resp, stop := r.handle(event.request)
				event.result <- resp
				if stop {
					r.closeAll()
					return
				}
			case outputEvent:
				r.handleOutput(event)
			case exitEvent:
				r.handleExit(event)
			case subscribeEvent:
				r.handleSubscribe(event)
			case unsubscribeEvent:
				r.handleUnsubscribe(event.id)
			}
		case <-ticker.C:
			if len(r.state.Clients) > 0 {
				r.broadcast()
			}
		}
	}
}

func (r *Reactor) handle(req protocol.Request) (protocol.Response, bool) {
	resp := protocol.Response{ID: req.ID, OK: true}
	var err error
	switch req.Op {
	case "ping":
	case "info":
		info := r.sessionInfo()
		resp.Session = &info
	case "list-windows":
		resp.Windows = r.windowInfos()
	case "list-panes":
		resp.Panes, err = r.paneInfos(req.Window)
	case "list-clients":
		resp.Clients = r.clientInfos()
	case "input":
		err = r.input(req.Window, req.Pane, []byte(req.Data))
	case "capture-pane":
		resp.Data, err = r.capture(req.Window, req.Pane)
	case "clear-history":
		err = r.clearHistory(req.Window, req.Pane)
	case "new-window":
		_, err = r.newWindow(req.Name, req.Data)
	case "split-pane":
		err = r.splitPane(req.Window, req.Horizontal, req.Data)
	case "select-window":
		err = r.selectWindow(req.Window)
	case "select-window-relative":
		err = r.selectWindowRelative(req.Direction)
	case "last-window":
		err = r.selectLastWindow()
	case "rename-window":
		err = r.renameWindow(req.Window, req.Name)
	case "kill-window":
		err = r.killWindow(req.Window)
	case "select-pane":
		err = r.selectPane(req.Window, req.Pane, req.Direction)
	case "swap-pane":
		err = r.swapPane(req.Window, req.SourcePane, req.Pane, req.Direction)
	case "kill-pane":
		err = r.killPane(req.Window, req.Pane)
	case "resize-pane":
		err = r.resizePane(req)
	case "zoom-pane":
		err = r.zoomPane(req.Window, req.Pane)
	case "select-layout":
		err = r.selectLayout(req.Window, req.Name)
	case "set-synchronize-panes":
		err = r.setSynchronize(req.Window, req.Enabled)
	case "resize-client":
		err = r.resize(req.Width, req.Height)
	case "display-message":
		if req.Data == "" {
			resp.Data = r.statusText()
		} else {
			r.state.Message = req.Data
		}
	case "set-buffer":
		resp.Data, err = r.setBuffer(req.Name, []byte(req.Data))
	case "get-buffer":
		resp.Data, err = r.getBuffer(req.Name)
	case "list-buffers":
		resp.Buffers = r.bufferInfos()
	case "delete-buffer":
		err = r.deleteBuffer(req.Name)
	case "paste-buffer":
		var data string
		data, err = r.getBuffer(req.Name)
		if err == nil {
			err = r.input(req.Window, req.Pane, []byte(data))
		}
	case "rename-session":
		if strings.TrimSpace(req.Name) == "" {
			err = fmt.Errorf("new session name is required")
		} else {
			r.state.Name = req.Name
		}
	case "shutdown":
		return resp, true
	default:
		err = fmt.Errorf("unsupported operation %q", req.Op)
	}
	if err != nil {
		resp.OK = false
		resp.Error = err.Error()
		return resp, false
	}
	if req.Op != "ping" && req.Op != "info" && !strings.HasPrefix(req.Op, "list-") && req.Op != "capture-pane" && req.Op != "get-buffer" {
		r.broadcast()
	}
	return resp, false
}

func (r *Reactor) handleOutput(event outputEvent) {
	window := r.state.Windows[event.window]
	if window == nil {
		return
	}
	pane := window.Panes[event.pane]
	if pane == nil {
		return
	}
	_, _ = pane.Term.Write(event.data)
	pane.History = append(pane.History, event.data...)
	if len(pane.History) > maxHistory {
		pane.History = append([]byte(nil), pane.History[len(pane.History)-maxHistory:]...)
	}
	r.broadcast()
}

func (r *Reactor) handleExit(event exitEvent) {
	if window := r.state.Windows[event.window]; window != nil {
		if pane := window.Panes[event.pane]; pane != nil {
			pane.Dead = true
		}
	}
	r.broadcast()
}

func (r *Reactor) handleSubscribe(event subscribeEvent) {
	r.state.NextClient++
	id := r.state.NextClient
	event.client.Frames = make(chan protocol.Frame, 4)
	if event.client.ID == "" {
		event.client.ID = fmt.Sprintf("client-%d", id)
	}
	event.client.Attached = time.Now()
	r.state.Clients[id] = event.client
	frame := r.frame()
	event.result <- subscription{id: id, frame: frame, ch: event.client.Frames}
}

func (r *Reactor) handleUnsubscribe(id uint64) {
	if client := r.state.Clients[id]; client != nil {
		delete(r.state.Clients, id)
		close(client.Frames)
	}
}

func (r *Reactor) broadcast() {
	frame := r.frame()
	for _, client := range r.state.Clients {
		select {
		case client.Frames <- frame:
		default:
			select {
			case <-client.Frames:
			default:
			}
			select {
			case client.Frames <- frame:
			default:
			}
		}
	}
}

func (r *Reactor) closeAll() {
	for _, window := range r.state.Windows {
		for _, pane := range window.Panes {
			closePane(pane)
		}
	}
	for id := range r.state.Clients {
		r.handleUnsubscribe(id)
	}
}

func sortedKeys[V any](values map[int]V) []int {
	keys := make([]int, 0, len(values))
	for key := range values {
		keys = append(keys, key)
	}
	sort.Ints(keys)
	return keys
}
