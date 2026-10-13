package service

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log/slog"
	"net"
	"os"
	"path/filepath"
	"sync"
	"time"

	"github.com/arcgolabs/logx"
	native "github.com/kardianos/service"
)

type broker struct {
	cfg           Config
	executable    string
	logger        *slog.Logger
	listener      net.Listener
	mu            sync.Mutex
	stopping      bool
	sessions      map[int]*session
	connections   map[net.Conn]struct{}
	workers       sync.WaitGroup
	acceptDone    chan struct{}
	stopRequested chan struct{}
	done          chan struct{}
	stopOnce      sync.Once
	stopErr       error
}

func newBroker(cfg Config) (*broker, error) {
	if err := validateConfig(cfg); err != nil {
		return nil, err
	}
	if err := requireRegularUser(); err != nil {
		return nil, err
	}
	if err := PrepareUserProfile(); err != nil {
		return nil, err
	}
	executable, err := os.Executable()
	if err != nil {
		return nil, fmt.Errorf("resolve service executable: %w", err)
	}
	name, err := effectiveName(cfg)
	if err != nil {
		return nil, err
	}
	cache, err := os.UserCacheDir()
	if err != nil {
		return nil, fmt.Errorf("resolve service log directory: %w", err)
	}
	logger, err := logx.New(logx.WithConsole(false), logx.WithRFC3339Time(),
		logx.WithFile(filepath.Join(cache, "terman", "logs", name+"-service.log")), logx.WithFileRotation(20, 7, 5))
	if err != nil {
		return nil, fmt.Errorf("create service logger: %w", err)
	}
	return &broker{cfg: cfg, executable: executable, logger: logger.With("component", "service", "tool", cfg.Tool, "service", name, "pid", os.Getpid()),
		sessions: make(map[int]*session), connections: make(map[net.Conn]struct{}),
		acceptDone: make(chan struct{}), stopRequested: make(chan struct{}, 1), done: make(chan struct{})}, nil
}

func (b *broker) Start(manager native.Service) error {
	address, err := Endpoint(b.cfg)
	if err != nil {
		return err
	}
	listener, err := listenEndpoint(address)
	if err != nil {
		return fmt.Errorf("listen for service sessions: %w", err)
	}
	b.listener = listener
	go b.accept()
	go func() {
		select {
		case <-b.stopRequested:
			if manager != nil {
				if err := manager.Stop(); err != nil {
					b.logger.Error("native service stop failed", "error", err)
					_ = b.Stop(nil)
				}
			} else {
				_ = b.Stop(nil)
			}
		case <-b.done:
		}
	}()
	b.logger.Info("service broker started")
	return nil
}

func (b *broker) accept() {
	defer close(b.acceptDone)
	slots := make(chan struct{}, 32)
	for {
		conn, err := b.listener.Accept()
		if err != nil {
			if !errors.Is(err, net.ErrClosed) {
				b.logger.Error("service accept failed", "error", err)
				b.requestStop()
			}
			return
		}
		select {
		case slots <- struct{}{}:
		default:
			_ = conn.Close()
			continue
		}
		b.mu.Lock()
		b.connections[conn] = struct{}{}
		b.workers.Add(1)
		b.mu.Unlock()
		go func() {
			defer b.workers.Done()
			defer func() { <-slots; b.mu.Lock(); delete(b.connections, conn); b.mu.Unlock(); _ = conn.Close() }()
			b.handle(conn)
		}()
	}
}

func (b *broker) requestStop() {
	select {
	case b.stopRequested <- struct{}{}:
	default:
	}
}

func (b *broker) handle(conn net.Conn) {
	_ = conn.SetDeadline(time.Now().Add(rpcTimeout))
	decoder := json.NewDecoder(io.LimitReader(conn, maxRequestBytes+1))
	decoder.DisallowUnknownFields()
	var req request
	var reply response
	err := decoder.Decode(&req)
	stop := false
	if err == nil {
		switch req.Operation {
		case "start":
			reply.PID, err = b.startSession(req.Launch)
		case "status":
			reply = b.status()
		case "stop":
			stop = true
		default:
			err = fmt.Errorf("unknown service operation %q", req.Operation)
		}
	}
	if err != nil {
		reply.Error = err.Error()
	}
	_ = json.NewEncoder(conn).Encode(reply)
	if stop {
		b.requestStop()
	}
}

// Stop rejects new sessions, asks every active child to stop gracefully, then
// kills remaining process trees and waits for the broker-owned child reapers.
func (b *broker) Stop(_ native.Service) error {
	b.stopOnce.Do(func() {
		defer close(b.done)
		defer logx.Close(b.logger)
		b.mu.Lock()
		b.stopping = true
		active := make([]*session, 0, len(b.sessions))
		for _, child := range b.sessions {
			active = append(active, child)
		}
		b.mu.Unlock()
		ctx, cancel := context.WithTimeout(context.Background(), 8*time.Second)
		defer cancel()
		var waits sync.WaitGroup
		var errorsMu sync.Mutex
		for _, child := range active {
			waits.Add(1)
			go func() {
				defer waits.Done()
				if err := b.stopSession(ctx, child); err != nil {
					errorsMu.Lock()
					b.stopErr = errors.Join(b.stopErr, err)
					errorsMu.Unlock()
				}
			}()
		}
		waits.Wait()
		if b.listener != nil {
			_ = b.listener.Close()
			<-b.acceptDone
			b.mu.Lock()
			for conn := range b.connections {
				_ = conn.Close()
			}
			b.mu.Unlock()
			b.workers.Wait()
		}
		b.logger.Info("service broker stopped")
	})
	return b.stopErr
}
