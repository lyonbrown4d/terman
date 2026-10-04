package server

import (
	"context"
	"errors"
	"fmt"
	"net"
	"os"
	"sync"
	"time"

	appconfig "github.com/lyonbrown4d/terman/tmux/internal/config"
	"github.com/lyonbrown4d/terman/tmux/internal/ipc"
	"github.com/lyonbrown4d/terman/tmux/internal/session"
	sessionstore "github.com/lyonbrown4d/terman/tmux/internal/store"
)

type Launch struct {
	Name     string
	Endpoint string
	Command  string
	Cols     int
	Rows     int
	Created  time.Time
}

type Store interface {
	Save(sessionstore.Record) error
	Remove(string) error
}

type ListenerFactory func() (net.Listener, error)
type ReactorFactory func(context.Context) (*session.Reactor, error)
type ServerFactory func(
	appconfig.Config,
	Store,
	ListenerFactory,
	ReactorFactory,
) *Server

type fileStore struct{}

func (fileStore) Save(record sessionstore.Record) error {
	return sessionstore.Save(record)
}

func (fileStore) Remove(name string) error {
	return sessionstore.Remove(name)
}

type Server struct {
	launch          Launch
	settings        appconfig.Config
	store           Store
	listenerFactory ListenerFactory
	reactorFactory  ReactorFactory

	ctx       context.Context
	cancel    context.CancelFunc
	listener  net.Listener
	reactor   *session.Reactor
	clients   sync.WaitGroup
	acceptErr chan error
	done      chan struct{}
	doneOnce  sync.Once
	stopOnce  sync.Once
	mu        sync.Mutex
	runErr    error
	stopErr   error
}

func newServer(
	launch Launch,
	settings appconfig.Config,
	store Store,
	listenerFactory ListenerFactory,
	reactorFactory ReactorFactory,
) *Server {
	return &Server{
		launch: launch, settings: settings, store: store,
		listenerFactory: listenerFactory,
		reactorFactory:  reactorFactory,
		acceptErr:       make(chan error, 1),
		done:            make(chan struct{}),
	}
}

func (s *Server) Start(ctx context.Context) error {
	s.ctx, s.cancel = context.WithCancel(context.WithoutCancel(ctx))
	listener, err := s.listenerFactory()
	if err != nil {
		return fmt.Errorf("listen on session endpoint: %w", err)
	}
	s.listener = listener
	reactor, err := s.reactorFactory(s.ctx)
	if err != nil {
		_ = listener.Close()
		_ = ipc.Cleanup(s.launch.Endpoint)
		return err
	}
	s.reactor = reactor
	record := sessionstore.Record{
		SchemaVersion: sessionstore.SchemaVersion,
		Name:          s.launch.Name,
		Endpoint:      s.launch.Endpoint,
		PID:           os.Getpid(),
		CreatedAt:     s.launch.Created,
	}
	if record.CreatedAt.IsZero() {
		record.CreatedAt = time.Now()
	}
	if err := s.store.Save(record); err != nil {
		s.cancel()
		s.reactor.Stop()
		<-s.reactor.Done()
		_ = listener.Close()
		_ = ipc.Cleanup(s.launch.Endpoint)
		return err
	}
	go s.acceptLoop()
	go s.monitor()
	return nil
}

func (s *Server) acceptLoop() {
	for {
		conn, err := s.listener.Accept()
		if err != nil {
			select {
			case s.acceptErr <- err:
			default:
			}
			return
		}
		s.clients.Add(1)
		go func() {
			defer s.clients.Done()
			handleClient(s.ctx, conn, s.reactor)
		}()
	}
}

func (s *Server) monitor() {
	select {
	case <-s.ctx.Done():
	case <-s.reactor.Done():
	case err := <-s.acceptErr:
		if !errors.Is(err, net.ErrClosed) {
			s.setError(fmt.Errorf("accept client: %w", err))
		}
	}
	s.signalDone()
}

func (s *Server) Stop(ctx context.Context) error {
	s.stopOnce.Do(func() {
		s.stopErr = s.stop(ctx)
	})
	return s.stopErr
}

func (s *Server) stop(ctx context.Context) error {
	if s.cancel != nil {
		s.cancel()
	}
	var failures []error
	if s.listener != nil {
		if err := s.listener.Close(); err != nil &&
			!errors.Is(err, net.ErrClosed) {
			failures = append(failures, err)
		}
	}
	if s.reactor != nil {
		s.reactor.Stop()
		select {
		case <-s.reactor.Done():
		case <-ctx.Done():
			failures = append(failures, ctx.Err())
		}
	}
	if err := s.waitClients(ctx); err != nil {
		failures = append(failures, err)
	}
	if err := ipc.Cleanup(s.launch.Endpoint); err != nil {
		failures = append(failures, err)
	}
	if err := s.store.Remove(s.launch.Name); err != nil {
		failures = append(failures, err)
	}
	s.signalDone()
	return errors.Join(failures...)
}

func (s *Server) waitClients(ctx context.Context) error {
	done := make(chan struct{})
	go func() {
		s.clients.Wait()
		close(done)
	}()
	select {
	case <-done:
		return nil
	case <-ctx.Done():
		return ctx.Err()
	}
}

func (s *Server) Done() <-chan struct{} {
	return s.done
}

func (s *Server) Err() error {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.runErr
}

func (s *Server) setError(err error) {
	s.mu.Lock()
	defer s.mu.Unlock()
	s.runErr = err
}

func (s *Server) signalDone() {
	s.doneOnce.Do(func() { close(s.done) })
}
