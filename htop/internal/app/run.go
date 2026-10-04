package app

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/gdamore/tcell/v2"
)

func Run(ctx context.Context, cfg Config) error {
	if cfg.Once {
		return printOnce(ctx, cfg)
	}
	screen, err := tcell.NewScreen()
	if err != nil {
		return fmt.Errorf("create screen: %w", err)
	}
	if err := screen.Init(); err != nil {
		return fmt.Errorf("initialize screen: %w", err)
	}
	screen.EnableMouse()
	screen.SetStyle(styleBase)
	runCtx, cancel := context.WithCancel(ctx)
	events := make(chan tcell.Event, 32)
	snapshots := make(chan Snapshot, 1)
	var workers sync.WaitGroup
	workers.Add(2)
	go pollEvents(runCtx, screen, events, &workers)
	go collectSnapshots(runCtx, cfg.Refresh, snapshots, &workers)
	defer func() {
		cancel()
		screen.PostEventWait(tcell.NewEventInterrupt(nil))
		workers.Wait()
		screen.DisableMouse()
		screen.Fini()
	}()
	state := newState(cfg)
	state.draw(screen)
	for {
		select {
		case <-ctx.Done():
			return nil
		case snapshot := <-snapshots:
			state.snapshot = snapshot
			state.normalize(screen)
			state.draw(screen)
		case event := <-events:
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

func pollEvents(ctx context.Context, screen tcell.Screen, output chan<- tcell.Event, workers *sync.WaitGroup) {
	defer workers.Done()
	for {
		event := screen.PollEvent()
		select {
		case <-ctx.Done():
			return
		case output <- event:
		}
	}
}

func collectSnapshots(ctx context.Context, refresh time.Duration, output chan Snapshot, workers *sync.WaitGroup) {
	defer workers.Done()
	ticker := time.NewTicker(refresh)
	defer ticker.Stop()
	collector := newCollector()
	for {
		snapshot := collector.collect(ctx)
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
