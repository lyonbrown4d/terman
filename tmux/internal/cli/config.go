package cli

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/arcgolabs/configx"
	appconfig "github.com/lyonbrown4d/terman/tmux/internal/config"
	"github.com/spf13/cobra"
	"github.com/spf13/pflag"
)

func loadConfig(
	root *cobra.Command,
	args []string,
) (appconfig.Config, []string, error) {
	defaults := appconfig.Default()
	bindConfigFlags(root.PersistentFlags(), defaults)
	remaining, err := consumeConfigFlags(root.PersistentFlags(), args)
	if err != nil {
		return appconfig.Config{}, nil, err
	}
	cfg, err := configx.Load[appconfig.Config](
		configx.WithTypedDefaults(defaults),
		configx.WithEnvPrefix("TERMAN_TMUX"),
		configx.WithEnvSeparator("__"),
		configx.WithFlagSet(root.PersistentFlags()),
		configx.WithArgsNameFunc(func(name string) string {
			return strings.ReplaceAll(name, "-", "_")
		}),
		configx.WithPriority(configx.SourceEnv, configx.SourceArgs),
	)
	if err != nil {
		return appconfig.Config{}, nil, fmt.Errorf("load tmux config: %w", err)
	}
	if err := cfg.Validate(); err != nil {
		return appconfig.Config{}, nil, err
	}
	return cfg, remaining, nil
}

func bindConfigFlags(flags *pflag.FlagSet, defaults appconfig.Config) {
	flags.Duration(
		"startup-timeout",
		defaults.StartupTimeout,
		"session server startup timeout",
	)
	flags.Duration(
		"shutdown-timeout",
		defaults.ShutdownTimeout,
		"server lifecycle shutdown timeout",
	)
	flags.Duration(
		"display-panes-timeout",
		defaults.DisplayPanesTimeout,
		"display-panes overlay timeout",
	)
	flags.String("log-level", defaults.LogLevel, "server log level")
	flags.Int(
		"terminal-cols",
		defaults.TerminalCols,
		"fallback terminal columns",
	)
	flags.Int(
		"terminal-rows",
		defaults.TerminalRows,
		"fallback terminal rows",
	)
}

func consumeConfigFlags(
	flags *pflag.FlagSet,
	args []string,
) ([]string, error) {
	remaining := make([]string, 0, len(args))
	for index := 0; index < len(args); index++ {
		token := args[index]
		if token == "--" {
			remaining = append(remaining, args[index:]...)
			break
		}
		if !strings.HasPrefix(token, "--") {
			remaining = append(remaining, token)
			continue
		}
		nameValue := strings.TrimPrefix(token, "--")
		name, value, hasValue := strings.Cut(nameValue, "=")
		if flags.Lookup(name) == nil {
			remaining = append(remaining, token)
			continue
		}
		if !hasValue {
			if index+1 >= len(args) {
				return nil, fmt.Errorf("flag --%s requires a value", name)
			}
			index++
			value = args[index]
		}
		if err := flags.Set(name, value); err != nil {
			return nil, fmt.Errorf("set config flag --%s: %w", name, err)
		}
	}
	return remaining, nil
}

func configFlagArgs(cfg appconfig.Config) []string {
	return []string{
		"--startup-timeout=" + cfg.StartupTimeout.String(),
		"--shutdown-timeout=" + cfg.ShutdownTimeout.String(),
		"--display-panes-timeout=" + cfg.DisplayPanesTimeout.String(),
		"--log-level=" + cfg.LogLevel,
		"--terminal-cols=" + strconv.Itoa(cfg.TerminalCols),
		"--terminal-rows=" + strconv.Itoa(cfg.TerminalRows),
	}
}
