package server

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"net"

	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
	"github.com/lyonbrown4d/terman/tmux/internal/session"
)

func handleClient(
	ctx context.Context,
	conn net.Conn,
	reactor *session.Reactor,
) {
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

func handleAttach(
	ctx context.Context,
	conn net.Conn,
	decoder *json.Decoder,
	reactor *session.Reactor,
	request protocol.Request,
) {
	client := &session.Client{
		ID: request.ClientID, Width: request.Width, Height: request.Height,
	}
	id, frame, frames, err := reactor.Subscribe(ctx, client)
	if err != nil {
		_ = json.NewEncoder(conn).Encode(protocol.Response{
			ID: request.ID, Error: err.Error(),
		})
		return
	}
	defer reactor.Unsubscribe(id)
	encoder := json.NewEncoder(conn)
	if err := encoder.Encode(protocol.Response{
		ID: request.ID, OK: true, Event: "attached", Frame: &frame,
	}); err != nil {
		return
	}
	requests := make(chan protocol.Request)
	readErr := make(chan struct{})
	go readAttachRequests(ctx, decoder, requests, readErr)
	for {
		select {
		case <-ctx.Done():
			return
		case <-readErr:
			return
		case response, ok := <-frames:
			if !ok || encoder.Encode(response) != nil {
				return
			}
			if response.Event == "detached" {
				return
			}
		case req := <-requests:
			if req.Op == "detach" {
				_ = encoder.Encode(protocol.Response{
					ID: req.ID, OK: true, Event: "detached",
				})
				return
			}
			if req.Op == "resize-client" {
				err := reactor.ResizeClient(ctx, id, req.Width, req.Height)
				if !encodeAttachResult(encoder, req.ID, err) {
					return
				}
				continue
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

func readAttachRequests(
	ctx context.Context,
	decoder *json.Decoder,
	requests chan<- protocol.Request,
	done chan<- struct{},
) {
	defer close(done)
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
}

func encodeAttachResult(
	encoder *json.Encoder,
	id uint64,
	err error,
) bool {
	response := protocol.Response{ID: id, OK: err == nil}
	if err != nil {
		response.Error = err.Error()
	}
	return encoder.Encode(response) == nil
}

func attachError(operation string, err error) error {
	return fmt.Errorf("%s attach client: %w", operation, err)
}
