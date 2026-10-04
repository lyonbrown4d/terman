package server

import (
	"bufio"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net"
	"os"
	"os/signal"
	"sync"
	"time"

	"github.com/lyonbrown4d/terman/tmux/internal/ipc"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
	"github.com/lyonbrown4d/terman/tmux/internal/session"
	"github.com/lyonbrown4d/terman/tmux/internal/store"
)

type Config struct {
	Name     string
	Endpoint string
	Command  string
	Cols     int
	Rows     int
	Created  time.Time
}

func Run(parent context.Context, config Config) error {
	listener, err := ipc.Listen(config.Endpoint)
	if err != nil {
		return fmt.Errorf("listen on session endpoint: %w", err)
	}
	defer listener.Close()
	defer ipc.Cleanup(config.Endpoint)
	defer store.Remove(config.Name)
	ctx, stop := signal.NotifyContext(parent, os.Interrupt)
	defer stop()
	reactor, err := session.New(ctx, config.Name, config.Command, config.Cols, config.Rows)
	if err != nil {
		return err
	}
	record := store.Record{
		SchemaVersion: store.SchemaVersion, Name: config.Name, Endpoint: config.Endpoint,
		PID: os.Getpid(), CreatedAt: config.Created,
	}
	if record.CreatedAt.IsZero() {
		record.CreatedAt = time.Now()
	}
	if err := store.Save(record); err != nil {
		reactor.Stop()
		return err
	}

	var clients sync.WaitGroup
	acceptErr := make(chan error, 1)
	go func() {
		for {
			conn, accept := listener.Accept()
			if accept != nil {
				acceptErr <- accept
				return
			}
			clients.Add(1)
			go func() {
				defer clients.Done()
				handleClient(ctx, conn, reactor)
			}()
		}
	}()

	select {
	case <-ctx.Done():
		reactor.Stop()
	case <-reactor.Done():
	case err := <-acceptErr:
		if !errors.Is(err, net.ErrClosed) {
			reactor.Stop()
			return fmt.Errorf("accept client: %w", err)
		}
	}
	_ = listener.Close()
	reactor.Stop()
	<-reactor.Done()
	clients.Wait()
	return nil
}

func handleClient(ctx context.Context, conn net.Conn, reactor *session.Reactor) {
	defer conn.Close()
	decoder := json.NewDecoder(bufio.NewReader(conn))
	var first protocol.Request
	if err := decoder.Decode(&first); err != nil {
		return
	}
	if first.Op != "attach" {
		resp, err := reactor.Call(ctx, first)
		if err != nil {
			resp = protocol.Response{ID: first.ID, Error: err.Error()}
		}
		_ = json.NewEncoder(conn).Encode(resp)
		return
	}
	handleAttach(ctx, conn, decoder, reactor, first)
}

func handleAttach(ctx context.Context, conn net.Conn, decoder *json.Decoder, reactor *session.Reactor, request protocol.Request) {
	client := &session.Client{ID: request.ClientID, Width: request.Width, Height: request.Height}
	id, frame, frames, err := reactor.Subscribe(ctx, client)
	if err != nil {
		_ = json.NewEncoder(conn).Encode(protocol.Response{ID: request.ID, Error: err.Error()})
		return
	}
	defer reactor.Unsubscribe(id)
	encoder := json.NewEncoder(conn)
	if err := encoder.Encode(protocol.Response{ID: request.ID, OK: true, Event: "attached", Frame: &frame}); err != nil {
		return
	}
	requests := make(chan protocol.Request)
	readErr := make(chan struct{})
	go func() {
		defer close(readErr)
		for {
			var req protocol.Request
			if decoder.Decode(&req) != nil {
				return
			}
			select {
			case requests <- req:
			case <-ctx.Done():
				return
			}
		}
	}()
	for {
		select {
		case <-ctx.Done():
			return
		case <-readErr:
			return
		case next, ok := <-frames:
			if !ok || encoder.Encode(protocol.Response{OK: true, Event: "frame", Frame: &next}) != nil {
				return
			}
		case req := <-requests:
			if req.Op == "detach" {
				_ = encoder.Encode(protocol.Response{ID: req.ID, OK: true, Event: "detached"})
				return
			}
			resp, callErr := reactor.Call(ctx, req)
			if callErr != nil {
				resp = protocol.Response{ID: req.ID, Error: callErr.Error()}
			}
			if encoder.Encode(resp) != nil {
				return
			}
		}
	}
}
