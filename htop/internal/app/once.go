package app

import (
	"context"
	"fmt"
	"os"
)

func printOnce(ctx context.Context, cfg Config, collector Collector) error {
	snapshot := collector.Collect(ctx)
	if snapshot.Warning != "" {
		fmt.Fprintln(os.Stdout, "warning:", snapshot.Warning)
	}
	fmt.Fprintf(os.Stdout, "Host: %s  Uptime: %s  CPU: %.1f%%  Memory: %s/%s\n",
		fallback(snapshot.Hostname, "-"), formatDuration(snapshot.Uptime), cpuAverage(snapshot.CPU),
		formatBytes(snapshot.MemoryUsed), formatBytes(snapshot.MemoryTotal))
	fmt.Fprintf(os.Stdout, "%-7s %-12s %7s %-10s %-10s %s\n", "PID", "USER", "CPU%", "MEM", "IO/s", "NAME")
	for _, row := range processRows(snapshot, cfg.Sort, cfg.Reverse, cfg.Filter) {
		fmt.Fprintf(os.Stdout, "%-7d %-12s %7.1f %-10s %-10s %s\n",
			row.PID, fit(row.User, 12), row.CPU, formatBytes(row.Memory),
			formatBytes(row.ReadRate+row.WriteRate), row.Name)
	}
	return nil
}
