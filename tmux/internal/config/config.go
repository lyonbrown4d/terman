package config

import (
	"fmt"
	"log/slog"
	"time"
)

type Config struct {
	StartupTimeout      time.Duration `json:"startup_timeout" koanf:"startup_timeout"`
	ShutdownTimeout     time.Duration `json:"shutdown_timeout" koanf:"shutdown_timeout"`
	DisplayPanesTimeout time.Duration `json:"display_panes_timeout" koanf:"display_panes_timeout"`
	LogLevel            string        `json:"log_level" koanf:"log_level"`
	TerminalCols        int           `json:"terminal_cols" koanf:"terminal_cols"`
	TerminalRows        int           `json:"terminal_rows" koanf:"terminal_rows"`
}

func Default() Config {
	return Config{
		StartupTimeout:      4 * time.Second,
		ShutdownTimeout:     5 * time.Second,
		DisplayPanesTimeout: 2 * time.Second,
		LogLevel:            "info",
		TerminalCols:        80,
		TerminalRows:        24,
	}
}

func (c Config) Validate() error {
	if c.StartupTimeout <= 0 {
		return fmt.Errorf("startup timeout must be positive")
	}
	if c.ShutdownTimeout <= 0 {
		return fmt.Errorf("shutdown timeout must be positive")
	}
	if c.DisplayPanesTimeout <= 0 {
		return fmt.Errorf("display panes timeout must be positive")
	}
	if c.TerminalCols < 2 || c.TerminalRows < 2 {
		return fmt.Errorf("terminal defaults must be at least 2x2")
	}
	var level slog.Level
	if err := level.UnmarshalText([]byte(c.LogLevel)); err != nil {
		return fmt.Errorf("invalid log level %q: %w", c.LogLevel, err)
	}
	return nil
}
