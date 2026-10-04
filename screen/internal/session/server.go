package session

import (
	"context"
	"encoding/json"
	"errors"
	"io"
	"net"
	"os"
	"sync"

	"github.com/lyonbrown4d/terman/screen/internal/proto"
	"github.com/lyonbrown4d/terman/screen/internal/store"
	"github.com/lyonbrown4d/terman/screen/internal/transport"
)

func Serve(ctx context.Context, config Config) error {
	listener, err := transport.Listen(config.Endpoint)
	if err != nil {
		return err
	}
	defer listener.Close()
	defer transport.Cleanup(config.Endpoint)

	owner, err := New(ctx, config)
	if err != nil {
		return err
	}
	if err := store.Save(owner.record()); err != nil {
		owner.cancel()
		return err
	}
	defer store.Delete(config.Name)

	var clients sync.WaitGroup
	go func() {
		<-owner.Done()
		_ = listener.Close()
	}()
	for {
		conn, acceptErr := listener.Accept()
		if acceptErr != nil {
			if errors.Is(acceptErr, net.ErrClosed) {
				break
			}
			return acceptErr
		}
		clients.Go(func() { serveConn(ctx, owner, conn) })
	}
	clients.Wait()
	return nil
}

func serveConn(ctx context.Context, owner *Owner, conn net.Conn) {
	defer conn.Close()
	decoder := json.NewDecoder(io.LimitReader(conn, 16<<20))
	encoder := json.NewEncoder(conn)
	var first proto.Request
	if err := decoder.Decode(&first); err != nil {
		return
	}
	if first.Type != "attach" {
		response, err := owner.Submit(ctx, first)
		if err != nil {
			response = rejected(err)
		}
		_ = encoder.Encode(response)
		return
	}
	queue, err := owner.Attach(ctx, first.ClientID, first.Mode, first.DetachExisting)
	if err != nil {
		_ = encoder.Encode(rejected(err))
		return
	}
	defer owner.Remove(first.ClientID)

	go func() {
		for {
			var request proto.Request
			if decoder.Decode(&request) != nil {
				return
			}
			if request.ClientID == "" {
				request.ClientID = first.ClientID
			}
			_, _ = owner.Submit(ctx, request)
		}
	}()
	for response := range queue {
		if encoder.Encode(response) != nil {
			return
		}
	}
}

func DefaultConfig(name, endpoint, command string, cols, rows int, login bool) Config {
	cwd, _ := os.Getwd()
	return Config{
		Name: name, Endpoint: endpoint, Command: command, Cwd: cwd,
		Cols: cols, Rows: rows, LoginShell: login,
	}
}
