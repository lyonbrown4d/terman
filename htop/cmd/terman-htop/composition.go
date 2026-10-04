package main

import (
	"context"
	"fmt"

	"github.com/arcgolabs/dix"
	"github.com/gdamore/tcell/v3"
	"github.com/lyonbrown4d/terman/htop/internal/app"
)

func runComposed(ctx context.Context, config app.Config) error {
	runner, err := composeRunner(config)
	if err != nil {
		return err
	}
	return runner.Run(ctx)
}

func composeRunner(config app.Config) (*app.Runner, error) {
	var runner *app.Runner
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
		dix.Invokes(dix.Invoke1(func(value *app.Runner) {
			runner = value
		})),
	)
	if _, err := dix.NewApp("terman-htop", module).Build(); err != nil {
		return nil, fmt.Errorf("compose application: %w", err)
	}
	if runner == nil {
		return nil, fmt.Errorf("compose application: runner was not resolved")
	}
	return runner, nil
}
