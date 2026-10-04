package app

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/gdamore/tcell/v3"
)

// ScreenFactory creates an uninitialized terminal screen.
type ScreenFactory func() (tcell.Screen, error)

// Runner owns the configured application dependencies.
type Runner struct {
	config    Config
	collector Collector
	screens   ScreenFactory
}

// NewRunner constructs an application runner without using a service locator.
func NewRunner(config Config, collector Collector, screens ScreenFactory) *Runner {
	return &Runner{config: config, collector: collector, screens: screens}
}

// Run preserves the package-level application API.
func Run(ctx context.Context, config Config) error {
	screens := func() (tcell.Screen, error) {
		return tcell.NewScreen()
	}
	return NewRunner(config, NewCollector(), screens).Run(ctx)
}

// Run starts either the one-shot renderer or the interactive TUI.
func (r *Runner) Run(ctx context.Context) error {
	if r == nil || r.collector == nil {
		return fmt.Errorf("collector is required")
	}
	config := r.config
	if config.Refresh <= 0 {
		config.Refresh = time.Second
	}
	if config.Once {
		return printOnce(ctx, config, r.collector)
	}
	if r.screens == nil {
		return fmt.Errorf("screen factory is required")
	}
	screen, err := r.screens()
	if err != nil {
		return fmt.Errorf("create screen: %w", err)
	}
	worker := func(ctx context.Context, refresh time.Duration, output chan Snapshot) {
		collectSnapshots(ctx, refresh, output, r.collector)
	}
	return runInteractive(ctx, config, screen, worker)
}

type snapshotWorker func(context.Context, time.Duration, chan Snapshot)

func runInteractive(
	ctx context.Context,
	config Config,
	screen tcell.Screen,
	worker snapshotWorker,
) error {
	if err := screen.Init(); err != nil {
		return fmt.Errorf("initialize screen: %w", err)
	}
	screen.EnableMouse()
	screen.SetStyle(styleBase)
	runCtx, cancel := context.WithCancel(ctx)
	snapshots := make(chan Snapshot, 1)
	var workers sync.WaitGroup
	defer func() {
		cancel()
		workers.Wait()
		screen.DisableMouse()
		screen.Fini()
	}()
	state := newState(config)
	state.draw(screen)
	workers.Add(1)
	go func() {
		defer workers.Done()
		worker(runCtx, config.Refresh, snapshots)
	}()
	for {
		select {
		case <-ctx.Done():
			return nil
		case snapshot := <-snapshots:
			state.snapshot = snapshot
			state.normalize(screen)
			state.draw(screen)
		case event, ok := <-screen.EventQ():
			if !ok {
				return nil
			}
			quit, eventErr := state.handleEvent(runCtx, screen, event)
			if eventErr != nil {
				state.setStatus(eventErr.Error())
			}
			if quit {
				return nil
			}
			state.normalize(screen)
			state.draw(screen)
		}
	}
}

func collectSnapshots(ctx context.Context, refresh time.Duration, output chan Snapshot, collector Collector) {
	ticker := time.NewTicker(refresh)
	defer ticker.Stop()
	for {
		snapshot := collector.Collect(ctx)
		select {
		case output <- snapshot:
		default:
			select {
			case <-output:
			default:
			}
			select {
			case output <- snapshot:
			case <-ctx.Done():
				return
			}
		}
		select {
		case <-ticker.C:
		case <-ctx.Done():
			return
		}
	}
}
