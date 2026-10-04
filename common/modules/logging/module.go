package logging

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"os"
	"path/filepath"
	"strings"

	"github.com/arcgolabs/dix"
	"github.com/arcgolabs/logx"
)

type Config struct {
	Component string
	Level     string
	Directory string
}

type Bundle struct {
	Module dix.Module
	Logger *slog.Logger
}

func New(config Config) (Bundle, error) {
	component := strings.TrimSpace(config.Component)
	if err := validateComponent(component); err != nil {
		return Bundle{}, err
	}
	level, err := resolveLevel(config.Level)
	if err != nil {
		return Bundle{}, err
	}
	directory, err := resolveDirectory(config.Directory)
	if err != nil {
		return Bundle{}, err
	}
	logger, err := logx.New(
		logx.WithLevel(level),
		logx.WithConsole(false),
		logx.WithFile(filepath.Join(directory, component+".log")),
		logx.WithFileRotation(20, 7, 5),
		logx.WithRFC3339Time(),
		logx.WithCaller(true),
	)
	if err != nil {
		return Bundle{}, fmt.Errorf("create %s logger: %w", component, err)
	}
	logger = logger.With("component", component, "pid", os.Getpid())
	module := dix.NewModule(
		"logging",
		dix.Providers(dix.Value(logger)),
		dix.Hooks(
			dix.OnStart[*slog.Logger](func(ctx context.Context, value *slog.Logger) error {
				value.InfoContext(ctx, "application started")
				return nil
			}, dix.LifecycleName("logging")),
			dix.OnStop[*slog.Logger](func(ctx context.Context, value *slog.Logger) error {
				value.InfoContext(ctx, "application stopping")
				return nil
			}, dix.LifecycleName("logging")),
		),
	)
	return Bundle{Module: module, Logger: logger}, nil
}

func (b Bundle) Close() error {
	return logx.Close(b.Logger)
}

func (b Bundle) Stop(ctx context.Context, runtime *dix.Runtime) error {
	if runtime == nil {
		return b.Close()
	}
	return errors.Join(runtime.Stop(ctx), b.Close())
}

func ValidateLevel(value string) error {
	_, err := resolveLevel(value)
	return err
}

func resolveLevel(value string) (slog.Level, error) {
	value = strings.TrimSpace(value)
	if value == "" {
		value = strings.TrimSpace(os.Getenv("TERMAN_LOG_LEVEL"))
	}
	if value == "" {
		value = "info"
	}
	level, err := logx.ParseLevel(value)
	if err != nil {
		return slog.LevelInfo, fmt.Errorf("parse log level: %w", err)
	}
	return level, nil
}

func resolveDirectory(value string) (string, error) {
	value = strings.TrimSpace(value)
	if value == "" {
		value = strings.TrimSpace(os.Getenv("TERMAN_LOG_DIR"))
	}
	if value != "" {
		return value, nil
	}
	cache, err := os.UserCacheDir()
	if err != nil {
		return "", fmt.Errorf("resolve user cache directory: %w", err)
	}
	return filepath.Join(cache, "terman", "logs"), nil
}

func validateComponent(value string) error {
	if value == "" {
		return fmt.Errorf("logging component is required")
	}
	for _, character := range value {
		valid := character == '-' || character == '_' || character == '.' ||
			character >= 'a' && character <= 'z' || character >= 'A' && character <= 'Z' ||
			character >= '0' && character <= '9'
		if !valid {
			return fmt.Errorf("invalid logging component %q", value)
		}
	}
	return nil
}
