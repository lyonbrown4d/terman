package app

import (
	"context"
	"time"

	"github.com/arcgolabs/configx"
	"github.com/lyonbrown4d/terman/screen/internal/session"
	"github.com/spf13/pflag"
)

type PersistentConfig struct {
	Defaults DefaultSettings  `koanf:"defaults"`
	Terminal TerminalSettings `koanf:"terminal"`
	Logging  LoggingSettings  `koanf:"logging"`
}

type DefaultSettings struct {
	LoginShell     bool   `koanf:"login" configx:"flag=login-shell,usage=start commands through a login shell"`
	HardcopyDir    string `koanf:"hardcopydir" configx:"flag=hardcopydir,usage=default hardcopy output directory"`
	HardcopyAppend bool   `koanf:"hardcopyappend" configx:"flag=hardcopy-append,usage=append hardcopy output"`
}

type TerminalSettings struct {
	Name       string `koanf:"name" configx:"flag=term,usage=TERM value for child terminals" validate:"required"`
	Cols       int    `koanf:"cols" configx:"flag=cols,usage=default terminal columns" validate:"min=20"`
	Rows       int    `koanf:"rows" configx:"flag=rows,usage=default terminal rows" validate:"min=5"`
	Scrollback int    `koanf:"scrollback" configx:"flag=scrollback,usage=scrollback line limit" validate:"min=0,max=100000"`
}

type LoggingSettings struct {
	File            string        `koanf:"file" configx:"flag=logfile,usage=screen log path pattern"`
	Enabled         bool          `koanf:"enabled" configx:"flag=deflog,usage=enable logging for new windows"`
	Timestamp       bool          `koanf:"timestamp" configx:"flag=logtstamp,usage=enable inactivity timestamps"`
	TimestampAfter  time.Duration `koanf:"after" configx:"flag=logtstamp-after,usage=inactivity before a log timestamp"`
	TimestampFormat string        `koanf:"format" configx:"flag=logtstamp-format,usage=log timestamp format"`
}

func defaultPersistentConfig() PersistentConfig {
	return PersistentConfig{
		Terminal: TerminalSettings{
			Name: "xterm-256color", Cols: 80, Rows: 24, Scrollback: 1000,
		},
		Logging: LoggingSettings{
			File: "screenlog.%n", TimestampAfter: 2 * time.Minute,
			TimestampFormat: "-- %Y-%m-%d %H:%M:%S --\n",
		},
	}
}

func bindConfigFlags(flags *pflag.FlagSet, defaults PersistentConfig) error {
	schema, err := configx.SchemaOf(defaults)
	if err != nil {
		return err
	}
	return schema.BindFlags(flags, configx.WithFlagNameSeparator("-"))
}

func loadPersistentConfig(
	ctx context.Context,
	flags *pflag.FlagSet,
	defaults PersistentConfig,
) (PersistentConfig, error) {
	return configx.LoadContext[PersistentConfig](
		ctx,
		configx.WithTypedDefaults(defaults),
		configx.WithEnvPrefix("TERMAN_SCREEN"),
		configx.WithFlagSet(flags),
		configx.WithPriority(configx.SourceEnv, configx.SourceArgs),
		configx.WithValidateLevel(configx.ValidateLevelStruct),
	)
}

func (c PersistentConfig) sessionSettings() session.Settings {
	return session.Settings{
		Term: c.Terminal.Name, Scrollback: c.Terminal.Scrollback,
		LoginShell:     c.Defaults.LoginShell,
		HardcopyDir:    c.Defaults.HardcopyDir,
		HardcopyAppend: c.Defaults.HardcopyAppend,
		Logfile:        c.Logging.File, Deflog: c.Logging.Enabled,
		LogTimestamp: c.Logging.Timestamp,
		LogAfter:     c.Logging.TimestampAfter,
		LogStamp:     c.Logging.TimestampFormat,
	}
}
