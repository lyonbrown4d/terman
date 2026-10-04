package client

import (
	"bufio"
	"context"
	"encoding/json"
	"fmt"
	"net"
	"time"

	"github.com/lyonbrown4d/terman/tmux/internal/ipc"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
	"github.com/lyonbrown4d/terman/tmux/internal/store"
)

func LiveRecords(clean bool) ([]store.Record, error) {
	records, err := store.Load()
	if err != nil {
		return nil, err
	}
	live := make([]store.Record, 0, len(records))
	now := time.Now()
	for _, record := range records {
		if ipc.Ping(record.Endpoint) {
			live = append(live, record)
			continue
		}
		if clean && now.Sub(record.UpdatedAt) > 3*time.Second {
			_ = store.Remove(record.Name)
			_ = ipc.Cleanup(record.Endpoint)
		}
	}
	return live, nil
}

func Find(name string) (store.Record, error) {
	records, err := LiveRecords(true)
	if err != nil {
		return store.Record{}, err
	}
	if name == "" && len(records) > 0 {
		return records[0], nil
	}
	for _, record := range records {
		if record.Name == name {
			return record, nil
		}
	}
	return store.Record{}, fmt.Errorf("session %q not found", name)
}

func Call(ctx context.Context, record store.Record, req protocol.Request) (protocol.Response, error) {
	return ipc.Call(ctx, record.Endpoint, req)
}

type Stream struct {
	Conn    net.Conn
	Encoder *json.Encoder
	Decoder *json.Decoder
}

func OpenStream(ctx context.Context, record store.Record) (*Stream, error) {
	conn, err := ipc.Dial(ctx, record.Endpoint)
	if err != nil {
		return nil, fmt.Errorf("connect attach stream: %w", err)
	}
	return &Stream{
		Conn: conn, Encoder: json.NewEncoder(conn),
		Decoder: json.NewDecoder(bufio.NewReader(conn)),
	}, nil
}

func (s *Stream) Close() error { return s.Conn.Close() }
