package cli

import (
	"context"
	"fmt"

	"github.com/lyonbrown4d/terman/common/modules/service"
	"github.com/lyonbrown4d/terman/tmux/internal/client"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
	"github.com/lyonbrown4d/terman/tmux/internal/store"
)

func ServiceConfig() service.Config {
	return service.Config{
		Tool: "terman-tmux", ServerArgument: "__server",
		Shutdown: shutdownManagedSession,
	}
}

func shutdownManagedSession(ctx context.Context, launch service.Launch) error {
	if launch.PID <= 0 {
		return fmt.Errorf("managed tmux session PID is required")
	}
	records, err := store.Load()
	if err != nil {
		return err
	}
	for _, record := range records {
		if record.PID == launch.PID {
			_, err := client.Call(ctx, record, protocol.Request{Op: "shutdown"})
			return err
		}
	}
	return fmt.Errorf("managed tmux session PID %d has no record", launch.PID)
}
