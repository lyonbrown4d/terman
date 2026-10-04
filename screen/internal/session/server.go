package session

import (
	"context"
	"encoding/json"
	"errors"
	"io"
	"net"
	"sync"

	"github.com/lyonbrown4d/terman/screen/internal/proto"
	"github.com/lyonbrown4d/terman/screen/internal/transport"
)

type Server struct {
	config   Config
	records  Store
	listener *Listener
	factory  SessionFactory

	owner *Owner
	done  chan struct{}
	err   error
}

func newServer(
	config Config,
	records Store,
	listener *Listener,
	factory SessionFactory,
) *Server {
	return &Server{
		config: config, records: records, listener: listener,
		factory: factory, done: make(chan struct{}),
	}
}

func (s *Server) Start(ctx context.Context) error {
	owner, err := s.factory(ctx, s.config)
	if err != nil {
		return errors.Join(err, s.closeListener())
	}
	s.owner = owner
	if err := s.records.Save(owner.record()); err != nil {
		owner.cancel()
		<-owner.Done()
		return errors.Join(err, s.closeListener())
	}
	go s.accept(ctx)
	return nil
}

func (s *Server) Stop(ctx context.Context) error {
	closeErr := s.closeListener()
	var waitErr error
	if s.owner != nil {
		s.owner.cancel()
		select {
		case <-s.done:
		case <-ctx.Done():
			waitErr = ctx.Err()
		}
	}
	name := s.config.Name
	if s.owner != nil {
		name = s.owner.config.Name
	}
	return errors.Join(waitErr, closeErr, s.records.Delete(name))
}

func (s *Server) closeListener() error {
	closeErr := s.listener.Close()
	if errors.Is(closeErr, net.ErrClosed) {
		closeErr = nil
	}
	return errors.Join(closeErr, transport.Cleanup(s.config.Endpoint))
}

func (s *Server) Done() <-chan struct{} { return s.done }

func (s *Server) Err() error { return s.err }

func (s *Server) accept(ctx context.Context) {
	defer close(s.done)
	var clients sync.WaitGroup
	go func() {
		<-s.owner.Done()
		_ = s.listener.Close()
	}()
	for {
		conn, err := s.listener.Accept()
		if err != nil {
			if !errors.Is(err, net.ErrClosed) {
				s.err = err
				s.owner.cancel()
			}
			break
		}
		clients.Go(func() { serveConn(ctx, s.owner, conn) })
	}
	clients.Wait()
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
