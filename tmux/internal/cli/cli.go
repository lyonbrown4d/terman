package cli

import (
	"context"
	"fmt"
	"strings"

	"github.com/lyonbrown4d/terman/common"
	"github.com/lyonbrown4d/terman/tmux/internal/client"
	appconfig "github.com/lyonbrown4d/terman/tmux/internal/config"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
	"github.com/lyonbrown4d/terman/tmux/internal/store"
	"github.com/spf13/cobra"
)

var settings = appconfig.Default()

func Execute(args []string) error {
	return ExecuteContext(context.Background(), args)
}

func ExecuteContext(ctx context.Context, args []string) error {
	root := &cobra.Command{
		Use:   "terman-tmux",
		Short: "native Windows/Linux terminal multiplexer",
		Long: "native Windows/Linux terminal multiplexer\n\n" +
			common.LocalizedMessage("service-about", nil) + "\n" +
			common.LocalizedMessage("service-usage", map[string]string{"tool": "terman-tmux"}),
		SilenceUsage:       true,
		SilenceErrors:      true,
		DisableFlagParsing: true,
		RunE: func(command *cobra.Command, values []string) error {
			return dispatch(command.Context(), values)
		},
	}
	cfg, remaining, err := loadConfig(root, args)
	if err != nil {
		return err
	}
	settings = cfg
	root.SetArgs(remaining)
	if len(remaining) == 1 && (remaining[0] == "--help" || remaining[0] == "-h") {
		return root.Help()
	}
	return root.ExecuteContext(ctx)
}

func runAttach(args []string) error {
	record, err := targetRecord(args)
	if err != nil {
		return err
	}
	return client.Attach(context.Background(), record)
}

func targetRecord(args []string) (store.Record, error) {
	target := optionAny(args, []string{"-t", "--target-session"}, "")
	session, _, _ := parseTarget(target)
	return client.Find(session)
}

func simple(args []string, request protocol.Request) error {
	record, err := targetRecord(args)
	if err != nil {
		return err
	}
	response, err := client.Call(context.Background(), record, request)
	if err == nil && response.Data != "" {
		fmt.Println(response.Data)
	}
	return err
}

func printData(args []string, request protocol.Request) error {
	record, err := targetRecord(args)
	if err != nil {
		return err
	}
	response, err := client.Call(context.Background(), record, request)
	if err != nil {
		return err
	}
	fmt.Print(response.Data)
	if response.Data != "" && !strings.HasSuffix(response.Data, "\n") {
		fmt.Println()
	}
	return nil
}
