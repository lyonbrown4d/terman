package main

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/arcgolabs/configx"
	"github.com/lyonbrown4d/terman/htop/internal/app"
	"github.com/spf13/cobra"
)

const defaultRefreshMS = 1000

type commandConfig struct {
	RefreshMS int    `json:"refresh_ms" koanf:"refresh_ms"`
	Once      bool   `json:"once" koanf:"once"`
	Sort      string `json:"sort" koanf:"sort"`
	Reverse   bool   `json:"reverse" koanf:"reverse"`
	Filter    string `json:"filter" koanf:"filter"`
}

type runFunc func(context.Context, app.Config) error

func newRootCommand(run runFunc) *cobra.Command {
	var loaded *app.Config
	command := &cobra.Command{
		Use:           "terman-htop",
		Short:         "Native Windows/Linux process monitor",
		Args:          cobra.NoArgs,
		SilenceErrors: true,
		SilenceUsage:  true,
		PreRunE: func(command *cobra.Command, _ []string) error {
			if loaded != nil {
				return fmt.Errorf("configuration already loaded")
			}
			config, err := loadCommandConfig(command)
			if err != nil {
				return err
			}
			loaded = &config
			return nil
		},
		RunE: func(command *cobra.Command, _ []string) error {
			if loaded == nil {
				return fmt.Errorf("configuration was not loaded")
			}
			return run(command.Context(), *loaded)
		},
	}
	flags := command.Flags()
	flags.Int("refresh-ms", defaultRefreshMS, "refresh interval in milliseconds (minimum 100)")
	flags.Bool("once", false, "print one snapshot and exit")
	flags.String("sort", "cpu", "sort by cpu, memory, io, pid, user, or name")
	flags.Bool("reverse", false, "reverse the selected sort")
	flags.String("filter", "", "show processes matching this text")
	return command
}

func loadCommandConfig(command *cobra.Command) (app.Config, error) {
	defaults := commandConfig{RefreshMS: defaultRefreshMS, Sort: "cpu"}
	loaded, err := configx.Load[commandConfig](
		configx.WithTypedDefaults(defaults),
		configx.WithEnvPrefix("TERMAN_HTOP"),
		configx.WithEnvSeparator("__"),
		configx.WithFlagSet(command.Flags()),
		configx.WithArgsNameFunc(flagConfigName),
		configx.WithPriority(configx.SourceEnv, configx.SourceArgs),
	)
	if err != nil {
		return app.Config{}, fmt.Errorf("load configuration: %w", err)
	}
	if loaded.RefreshMS < 100 {
		return app.Config{}, fmt.Errorf("refresh-ms must be at least 100")
	}
	sortKey, err := app.ParseSortKey(loaded.Sort)
	if err != nil {
		return app.Config{}, err
	}
	return app.Config{
		Refresh: time.Duration(loaded.RefreshMS) * time.Millisecond,
		Once:    loaded.Once,
		Sort:    sortKey,
		Reverse: loaded.Reverse,
		Filter:  loaded.Filter,
	}, nil
}

func flagConfigName(name string) string {
	return strings.ReplaceAll(strings.ToLower(name), "-", "_")
}
