package main

import (
	"context"
	"fmt"
	"os"
	"os/signal"
	"syscall"

	"github.com/lyonbrown4d/terman/common"
	"github.com/lyonbrown4d/terman/common/modules/service"
	"github.com/lyonbrown4d/terman/tmux/internal/cli"
)

func main() {
	if err := service.PrepareUserProfile(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()
	if handled, err := service.Handle(ctx, cli.ServiceConfig(), os.Args[1:]); handled {
		if err != nil {
			fmt.Fprintln(os.Stderr, err)
			os.Exit(1)
		}
		return
	}
	if len(os.Args) == 2 && (os.Args[1] == "--version" || os.Args[1] == "-V") {
		fmt.Println("terman-tmux " + common.Version)
		return
	}
	if err := cli.ExecuteContext(ctx, os.Args[1:]); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
