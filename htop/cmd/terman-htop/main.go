package main

import (
	"context"
	"errors"
	"fmt"
	"os"
	"os/signal"
)

func main() {
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt)
	defer stop()
	command := newRootCommand(runComposed)
	if err := command.ExecuteContext(ctx); err != nil && !errors.Is(err, context.Canceled) {
		fmt.Fprintln(os.Stderr, "terman-htop:", err)
		os.Exit(1)
	}
}
