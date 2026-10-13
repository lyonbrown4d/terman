package app

import (
	"context"
	"fmt"

	"github.com/lyonbrown4d/terman/common/modules/service"
	"github.com/lyonbrown4d/terman/screen/internal/client"
	"github.com/lyonbrown4d/terman/screen/internal/proto"
	"github.com/lyonbrown4d/terman/screen/internal/store"
)

func ServiceConfig() service.Config {
	return service.Config{
		Tool: "terman-screen", ServerArgument: "--__screen-server",
		Shutdown: shutdownManagedSession,
	}
}

func shutdownManagedSession(ctx context.Context, launch service.Launch) error {
	if launch.PID <= 0 {
		return fmt.Errorf("managed screen session PID is required")
	}
	records, err := store.List()
	if err != nil {
		return err
	}
	for _, record := range records {
		if record.PID == launch.PID {
			_, err := client.Request(ctx, record.Endpoint, proto.Request{
				Type: "command", Command: "quit",
			})
			return err
		}
	}
	return fmt.Errorf("managed screen session PID %d has no record", launch.PID)
}
