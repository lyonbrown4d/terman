// Package service manages per-user session brokers and native service registration.
package service

import (
	"context"
	"crypto/sha256"
	"fmt"
	"os"
	"strings"

	"github.com/arcgolabs/dix"
)

// Config describes a tool's fixed server entry point and graceful session shutdown.
type Config struct {
	Tool           string
	ServerArgument string
	Shutdown       func(context.Context, Launch) error
	// ServiceName optionally selects Tool or Tool followed by a safe suffix.
	ServiceName string
}

// Launch describes a session server. The broker fills PID before calling Shutdown.
type Launch struct {
	Name string
	Cwd  string
	Args []string
	PID  int
}

// Module supplies the service configuration to the application's dix container.
func Module(cfg Config) dix.Module {
	return dix.NewModule("service", dix.Providers(dix.Value(cfg)),
		dix.Hooks(dix.OnStart[Config](func(_ context.Context, cfg Config) error { return validateConfig(cfg) })))
}

func validateConfig(cfg Config) error {
	if !safeName(cfg.Tool, 80) {
		return serviceError("service-invalid-name", map[string]string{"name": cfg.Tool})
	}
	if cfg.ServerArgument == "" || strings.ContainsAny(cfg.ServerArgument, "\x00\r\n") {
		return fmt.Errorf("service server argument is required and must contain no control characters")
	}
	_, err := effectiveName(cfg)
	return err
}

func safeName(value string, maximum int) bool {
	if len(value) == 0 || len(value) > maximum {
		return false
	}
	for _, c := range value {
		if !(c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z' || c >= '0' && c <= '9' || c == '-' || c == '_') {
			return false
		}
	}
	return true
}

// Endpoint returns the current user's tool-specific native IPC address.
func Endpoint(cfg Config) (string, error) {
	if err := validateConfig(cfg); err != nil {
		return "", err
	}
	identity, err := UserIdentity()
	if err != nil {
		return "", err
	}
	name, err := effectiveName(cfg)
	if err != nil {
		return "", err
	}
	return endpoint(name, identity)
}

func identityHash(identity string) string {
	sum := sha256.Sum256([]byte(identity))
	return fmt.Sprintf("%x", sum[:12])
}

func effectiveName(cfg Config) (string, error) {
	name := cfg.ServiceName
	if name == "" {
		name = cfg.Tool
		if suffix := os.Getenv("TERMAN_SERVICE_INSTANCE"); suffix != "" {
			if !safeName(suffix, 40) {
				return "", fmt.Errorf("invalid service instance suffix")
			}
			name += "-" + suffix
		}
	}
	if name != cfg.Tool && (!strings.HasPrefix(name, cfg.Tool+"-") || !safeName(strings.TrimPrefix(name, cfg.Tool+"-"), 40)) {
		return "", serviceError("service-invalid-name", map[string]string{"name": name})
	}
	return name, nil
}

// PrepareUserProfile restores missing profile variables for a regular Windows
// service account before application logging and session paths are initialized.
// Existing variables, including deliberately isolated test profiles, are kept.
func PrepareUserProfile() error { return restoreUserProfile() }
