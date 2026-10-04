package main

import (
	"context"
	"errors"
	"flag"
	"fmt"
	"os"
	"os/signal"
	"time"

	"github.com/lyonbrown4d/terman/htop/internal/app"
)

func main() {
	cfg, err := parseArgs(os.Args[1:])
	if err != nil {
		fmt.Fprintln(os.Stderr, "terman-htop:", err)
		os.Exit(2)
	}
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt)
	defer stop()
	if err := app.Run(ctx, cfg); err != nil && !errors.Is(err, context.Canceled) {
		fmt.Fprintln(os.Stderr, "terman-htop:", err)
		os.Exit(1)
	}
}

func parseArgs(args []string) (app.Config, error) {
	var cfg app.Config
	var refreshMS int
	var sortName string
	flags := flag.NewFlagSet("terman-htop", flag.ContinueOnError)
	flags.SetOutput(os.Stderr)
	flags.IntVar(&refreshMS, "refresh-ms", 1000, "refresh interval in milliseconds (minimum 100)")
	flags.BoolVar(&cfg.Once, "once", false, "print one snapshot and exit")
	flags.StringVar(&sortName, "sort", "cpu", "sort by cpu, memory, io, pid, or name")
	flags.BoolVar(&cfg.Reverse, "reverse", false, "reverse the selected sort")
	flags.StringVar(&cfg.Filter, "filter", "", "show processes matching this text")
	if err := flags.Parse(args); err != nil {
		return cfg, err
	}
	if flags.NArg() != 0 {
		return cfg, fmt.Errorf("unexpected arguments: %v", flags.Args())
	}
	if refreshMS < 100 {
		return cfg, fmt.Errorf("--refresh-ms must be at least 100")
	}
	sortKey, err := app.ParseSortKey(sortName)
	if err != nil {
		return cfg, err
	}
	cfg.Refresh = time.Duration(refreshMS) * time.Millisecond
	cfg.Sort = sortKey
	return cfg, nil
}
