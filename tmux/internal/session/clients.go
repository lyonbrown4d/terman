package session

import (
	"fmt"
	"time"

	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
)

func (r *Reactor) handleSubscribe(event subscribeEvent) {
	r.state.NextClient++
	id := r.state.NextClient
	client := event.client
	client.Frames = make(chan protocol.Response, 4)
	client.ID = r.uniqueClientID(client.ID, id)
	client.Attached = time.Now()
	if client.Width < 2 {
		client.Width = r.state.Cols
	}
	if client.Height < 2 {
		client.Height = r.state.Rows
	}
	r.state.Clients[id] = client
	r.reconcileClientSize()
	frame := r.frame()
	event.result <- subscription{id: id, frame: frame, ch: client.Frames}
}

func (r *Reactor) uniqueClientID(candidate string, id uint64) string {
	if candidate == "" {
		candidate = "client"
	}
	base := candidate
	for suffix := 1; ; suffix++ {
		used := false
		for _, client := range r.state.Clients {
			if client.ID == candidate {
				used = true
				break
			}
		}
		if !used {
			return candidate
		}
		candidate = fmt.Sprintf("%s-%d", base, suffix+1)
	}
}

func (r *Reactor) handleUnsubscribe(id uint64) {
	client := r.state.Clients[id]
	if client == nil {
		return
	}
	delete(r.state.Clients, id)
	close(client.Frames)
	r.reconcileClientSize()
	r.broadcast()
}

func (r *Reactor) handleResizeClient(event resizeClientEvent) {
	client := r.state.Clients[event.id]
	if client == nil {
		event.result <- fmt.Errorf("attached client not found")
		return
	}
	if event.width < 2 || event.height < 2 {
		event.result <- fmt.Errorf("terminal size must be at least 2x2")
		return
	}
	client.Width, client.Height = event.width, event.height
	r.reconcileClientSize()
	r.broadcast()
	event.result <- nil
}

func (r *Reactor) resizeNamedClient(clientID string, width, height int) error {
	if width < 2 || height < 2 {
		return fmt.Errorf("terminal size must be at least 2x2")
	}
	var target *Client
	for _, client := range r.state.Clients {
		if clientID == "" || client.ID == clientID {
			if target != nil {
				return fmt.Errorf("client target is required when multiple clients are attached")
			}
			target = client
		}
	}
	if target == nil {
		return fmt.Errorf("client %q not found", clientID)
	}
	target.Width, target.Height = width, height
	r.reconcileClientSize()
	return nil
}

func (r *Reactor) reconcileClientSize() {
	if len(r.state.Clients) == 0 {
		return
	}
	cols, rows := int(^uint(0)>>1), int(^uint(0)>>1)
	for _, client := range r.state.Clients {
		cols = min(cols, client.Width)
		rows = min(rows, client.Height)
	}
	if cols == r.state.Cols && rows == r.state.Rows {
		return
	}
	_ = r.resize(cols, rows)
}

func (r *Reactor) detachClients(clientID string, all bool) (int, error) {
	if clientID == "" && !all {
		return 0, fmt.Errorf("client target is required")
	}
	ids := make([]uint64, 0, len(r.state.Clients))
	for id, client := range r.state.Clients {
		if all || client.ID == clientID {
			ids = append(ids, id)
		}
	}
	if len(ids) == 0 && !all {
		return 0, fmt.Errorf("client %q not found", clientID)
	}
	for _, id := range ids {
		client := r.state.Clients[id]
		r.sendClient(client, protocol.Response{OK: true, Event: "detached"})
		delete(r.state.Clients, id)
		close(client.Frames)
	}
	r.reconcileClientSize()
	return len(ids), nil
}

func (r *Reactor) sendClient(client *Client, response protocol.Response) {
	select {
	case client.Frames <- response:
	default:
		select {
		case <-client.Frames:
		default:
		}
		client.Frames <- response
	}
}
