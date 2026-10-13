package main

import (
	"fmt"
	"os"

	"github.com/lyonbrown4d/terman/common"
	"github.com/lyonbrown4d/terman/tmux/internal/cli"
)

func main() {
	if len(os.Args) == 2 && (os.Args[1] == "--version" || os.Args[1] == "-V") {
		fmt.Println("terman-tmux " + common.Version)
		return
	}
	if err := cli.Execute(os.Args[1:]); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
