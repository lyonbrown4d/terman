package session

import (
	"context"
	"errors"
	"net"
	"time"

	"github.com/arcgolabs/dix"
	"github.com/lyonbrown4d/terman/screen/internal/store"
	"github.com/lyonbrown4d/terman/screen/internal/transport"
)

type Store interface {
	Save(store.Record) error
	Delete(string) error
}

type fileStore struct{}

func (fileStore) Save(record store.Record) error { return store.Save(record) }

func (fileStore) Delete(name string) error { return store.Delete(name) }

type Listener struct {
	net.Listener
}

type SessionFactory func(context.Context, Config) (*Owner, error)

func provideListener(config Config) (*Listener, error) {
	listener, err := transport.Listen(config.Endpoint)
	if err != nil {
		return nil, err
	}
	return &Listener{Listener: listener}, nil
}

func serverModule(config Config) dix.Module {
	return dix.NewModule(
		"screen-server",
		dix.Providers(
			dix.Value(config),
			dix.Provider[Store](func() Store { return fileStore{} }),
			dix.Value(SessionFactory(New)),
			dix.ProviderErr1[*Listener, Config](provideListener),
			dix.Provider4[*Server, Config, Store, *Listener, SessionFactory](newServer),
		),
		dix.Hooks(
			dix.OnStart[*Server](
				func(ctx context.Context, server *Server) error {
					return server.Start(ctx)
				},
			),
			dix.OnStop[*Server](
				func(ctx context.Context, server *Server) error {
					return server.Stop(ctx)
				},
			),
		),
	)
}

func Serve(ctx context.Context, config Config) error {
	app := dix.New("terman-screen-server", dix.Modules(serverModule(config)))
	if err := app.ValidateContext(ctx); err != nil {
		return err
	}
	runtime, err := app.Start(ctx)
	if err != nil {
		return err
	}
	server, err := runtime.Container().Resolve[*Server]()
	if err != nil {
		return errors.Join(err, stopRuntime(runtime))
	}

	var serveErr error
	select {
	case <-ctx.Done():
	case <-server.Done():
		serveErr = server.Err()
	}
	return errors.Join(serveErr, stopRuntime(runtime))
}

func stopRuntime(runtime *dix.Runtime) error {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	return runtime.Stop(ctx)
}
