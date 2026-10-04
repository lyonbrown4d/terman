package session

import (
	"fmt"
	"sort"
	"strconv"
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
			if r.handleEvent(raw) {
				r.closeAll()
				return
			}
		case <-ticker.C:
			if len(r.state.Clients) > 0 {
				r.broadcast()
			}
		}
	}
}

func (r *Reactor) handleEvent(raw any) bool {
	switch event := raw.(type) {
	case requestEvent:
		resp, stop := r.handle(event.request)
		event.result <- resp
		return stop
	case outputEvent:
		r.handleOutput(event)
	case exitEvent:
		r.handleExit(event)
	case subscribeEvent:
		r.handleSubscribe(event)
	case unsubscribeEvent:
		r.handleUnsubscribe(event.id)
	case resizeClientEvent:
		r.handleResizeClient(event)
	}
	return false
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
		if err == nil {
			r.state.DisplayPanesUntil = time.Time{}
		}
	case "swap-pane":
		err = r.swapPane(
			req.Window,
			req.SourcePane,
			req.Pane,
			req.Direction,
		)
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
		err = r.resizeNamedClient(req.ClientID, req.Width, req.Height)
	case "detach-client":
		var count int
		count, err = r.detachClients(req.ClientID, req.All)
		resp.Data = strconv.Itoa(count)
	case "display-panes":
		r.state.DisplayPanesUntil = time.Now().Add(r.displayPanesTimeout)
	case "cancel-display-panes":
		r.state.DisplayPanesUntil = time.Time{}
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
	if shouldBroadcast(req.Op) {
		r.broadcast()
	}
	return resp, false
}

func shouldBroadcast(operation string) bool {
	if operation == "ping" || operation == "info" ||
		operation == "capture-pane" || operation == "get-buffer" {
		return false
	}
	return !strings.HasPrefix(operation, "list-")
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
		pane.History = append(
			[]byte(nil),
			pane.History[len(pane.History)-maxHistory:]...,
		)
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

func (r *Reactor) broadcast() {
	frame := r.frame()
	response := protocol.Response{OK: true, Event: "frame", Frame: &frame}
	for _, client := range r.state.Clients {
		r.sendClient(client, response)
	}
}

func (r *Reactor) closeAll() {
	for _, window := range r.state.Windows {
		for _, pane := range window.Panes {
			closePane(pane)
		}
	}
	for id, client := range r.state.Clients {
		delete(r.state.Clients, id)
		close(client.Frames)
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
