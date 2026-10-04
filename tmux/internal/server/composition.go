package server

import (
	"context"
	"errors"
	"net"
	"os"
	"os/signal"
	"time"

	"github.com/arcgolabs/dix"
	commonlogging "github.com/lyonbrown4d/terman/common/modules/logging"
	appconfig "github.com/lyonbrown4d/terman/tmux/internal/config"
	"github.com/lyonbrown4d/terman/tmux/internal/ipc"
	"github.com/lyonbrown4d/terman/tmux/internal/session"
)

func Run(
	parent context.Context,
	launch Launch,
	settings appconfig.Config,
) error {
	ctx, cancel := signal.NotifyContext(parent, os.Interrupt)
	defer cancel()
	logs, err := commonlogging.New(commonlogging.Config{Component: "tmux-" + launch.Name, Level: settings.LogLevel})
	if err != nil {
		return err
	}
	app := compose(launch, settings, logs)
	runtime, err := app.Start(ctx)
	if err != nil {
		return errors.Join(err, logs.Close())
	}
	server, err := runtime.Container().Resolve[*Server]()
	if err != nil {
		stopRuntime(runtime, settings.ShutdownTimeout, logs)
		return err
	}
	select {
	case <-ctx.Done():
	case <-server.Done():
	}
	runErr := server.Err()
	if runErr != nil {
		logs.Logger.ErrorContext(ctx, "tmux session server stopped", "session", launch.Name, "error", runErr)
	}
	stopErr := stopRuntime(runtime, settings.ShutdownTimeout, logs)
	return errors.Join(runErr, stopErr)
}

func compose(launch Launch, settings appconfig.Config, logs commonlogging.Bundle) *dix.App {
	listenerFactory := ListenerFactory(func() (net.Listener, error) {
		return ipc.Listen(launch.Endpoint)
	})
	reactorFactory := ReactorFactory(func(ctx context.Context) (*session.Reactor, error) {
		return session.New(
			ctx,
			launch.Name,
			launch.Command,
			launch.Cols,
			launch.Rows,
			session.WithDisplayPanesTimeout(settings.DisplayPanesTimeout),
		)
	})
	serverFactory := ServerFactory(func(
		cfg appconfig.Config,
		store Store,
		listener ListenerFactory,
		reactor ReactorFactory,
	) *Server {
		return newServer(launch, cfg, store, listener, reactor)
	})
	module := dix.NewModule(
		"server",
		dix.Providers(
			dix.Value(settings),
			dix.Provider0[Store](func() Store { return fileStore{} }),
			dix.Value(listenerFactory),
			dix.Value(reactorFactory),
			dix.Value(serverFactory),
			dix.Provider5[
				*Server,
				ServerFactory,
				appconfig.Config,
				Store,
				ListenerFactory,
				ReactorFactory,
			](func(
				factory ServerFactory,
				cfg appconfig.Config,
				store Store,
				listener ListenerFactory,
				reactor ReactorFactory,
			) *Server {
				return factory(cfg, store, listener, reactor)
			}),
		),
		dix.Hooks(
			dix.OnStart[*Server](
				func(ctx context.Context, server *Server) error {
					return server.Start(ctx)
				},
				dix.LifecycleName("server"),
			),
			dix.OnStop[*Server](
				func(ctx context.Context, server *Server) error {
					return server.Stop(ctx)
				},
				dix.LifecycleName("server"),
			),
		),
	)
	return dix.New(
		"terman-tmux-server",
		dix.WithLogger(logs.Logger),
		dix.Modules(logs.Module, module),
	)
}

func stopRuntime(runtime *dix.Runtime, timeout time.Duration, logs commonlogging.Bundle) error {
	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()
	return logs.Stop(ctx, runtime)
}
