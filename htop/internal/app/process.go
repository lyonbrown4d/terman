package app

import (
	"context"

	"github.com/shirou/gopsutil/v4/mem"
	gprocess "github.com/shirou/gopsutil/v4/process"
)

var memoryProvider = mem.VirtualMemoryWithContext

func newProcess(ctx context.Context, pid int32) (*gprocess.Process, error) {
	return gprocess.NewProcessWithContext(ctx, pid)
}

func loadEnvironment(ctx context.Context, pid int32) ([]string, error) {
	process, err := newProcess(ctx, pid)
	if err != nil {
		return nil, err
	}
	return process.EnvironWithContext(ctx)
}
