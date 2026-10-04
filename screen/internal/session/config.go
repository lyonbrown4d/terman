package session

import (
	"os"
	"time"
)

type Settings struct {
	Term           string
	Scrollback     int
	LoginShell     bool
	HardcopyDir    string
	HardcopyAppend bool
	Logfile        string
	Deflog         bool
	LogTimestamp   bool
	LogAfter       time.Duration
	LogStamp       string
}

type Config struct {
	Name, Endpoint, Command, Cwd string
	Cols, Rows                   int
	Settings
}

func DefaultConfig(
	name, endpoint, command string,
	cols, rows int,
	settings Settings,
) Config {
	cwd, _ := os.Getwd()
	return normalizeConfig(Config{
		Name: name, Endpoint: endpoint, Command: command, Cwd: cwd,
		Cols: cols, Rows: rows, Settings: settings,
	})
}

func normalizeConfig(config Config) Config {
	config.Cols = max(config.Cols, 20)
	config.Rows = max(config.Rows, 5)
	if config.Term == "" {
		config.Term = "xterm-256color"
	}
	if config.Scrollback < 0 {
		config.Scrollback = 1000
	}
	if config.Logfile == "" {
		config.Logfile = "screenlog.%n"
	}
	if config.LogAfter <= 0 {
		config.LogAfter = 2 * time.Minute
	}
	if config.LogStamp == "" {
		config.LogStamp = "-- %Y-%m-%d %H:%M:%S --\n"
	}
	return config
}
