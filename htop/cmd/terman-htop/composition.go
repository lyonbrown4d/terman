package main

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/arcgolabs/dix"
	"github.com/gdamore/tcell/v3"
	commonlogging "github.com/lyonbrown4d/terman/common/modules/logging"
	"github.com/lyonbrown4d/terman/htop/internal/app"
)

func runComposed(ctx context.Context, config app.Config) error {
	logs, err := commonlogging.New(commonlogging.Config{Component: "htop"})
	if err != nil {
		return err
	}
	runtime, err := composeApplication(config, logs).Start(ctx)
	if err != nil {
		return errors.Join(err, logs.Close())
	}
	runner, err := runtime.Container().Resolve[*app.Runner]()
	if err != nil {
		return errors.Join(err, stopRuntime(runtime, logs))
	}
	runErr := runner.Run(ctx)
	if runErr != nil {
		logs.Logger.ErrorContext(ctx, "htop stopped with an error", "error", runErr)
	}
	return errors.Join(runErr, stopRuntime(runtime, logs))
}

func composeApplication(config app.Config, logs commonlogging.Bundle) *dix.App {
	module := dix.NewModule(
		"terman-htop",
		dix.Providers(
			dix.Value(config),
			dix.Provider[app.Collector](app.NewCollector),
			dix.Provider[app.ScreenFactory](func() app.ScreenFactory {
				return func() (tcell.Screen, error) {
					return tcell.NewScreen()
				}
			}),
			dix.Provider3[*app.Runner, app.Config, app.Collector, app.ScreenFactory](app.NewRunner),
		),
	)
	return dix.New(
		"terman-htop",
		dix.WithLogger(logs.Logger),
		dix.Modules(logs.Module, module),
	)
}

func stopRuntime(runtime *dix.Runtime, logs commonlogging.Bundle) error {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	if err := logs.Stop(ctx, runtime); err != nil {
		return fmt.Errorf("stop application: %w", err)
	}
	return nil
}
