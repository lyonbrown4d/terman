package processtest

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"time"
)

func Build(
	ctx context.Context,
	directory string,
	output string,
	packagePath string,
	env []string,
) ([]byte, error) {
	command := exec.CommandContext(ctx, "go", "build", "-o", output, packagePath)
	command.Dir = directory
	command.Env = Environment(os.Environ(), env...)
	data, err := command.CombinedOutput()
	if err != nil {
		return data, fmt.Errorf("build %s: %w", packagePath, err)
	}
	return data, nil
}

func Run(
	ctx context.Context,
	directory string,
	executable string,
	env []string,
	args ...string,
) ([]byte, error) {
	command := exec.CommandContext(ctx, executable, args...)
	command.Dir = directory
	command.Env = Environment(os.Environ(), env...)
	return command.CombinedOutput()
}

func Environment(base []string, values ...string) []string {
	result := append([]string(nil), base...)
	for _, value := range values {
		key, _, found := strings.Cut(value, "=")
		if !found {
			continue
		}
		matched := false
		for index, current := range result {
			currentKey, _, _ := strings.Cut(current, "=")
			if strings.EqualFold(currentKey, key) {
				result[index] = value
				matched = true
				break
			}
		}
		if !matched {
			result = append(result, value)
		}
	}
	return result
}

func Wait(ctx context.Context, interval time.Duration, condition func() bool) bool {
	ticker := time.NewTicker(interval)
	defer ticker.Stop()
	for {
		if condition() {
			return true
		}
		select {
		case <-ctx.Done():
			return false
		case <-ticker.C:
		}
	}
}

func ExecutableName(name string) string {
	if runtime.GOOS == "windows" {
		return name + ".exe"
	}
	return name
}

func GlobalGoEnvironment() []string {
	cache := os.Getenv("GOCACHE")
	if cache == "" {
		cache = filepath.Join(os.TempDir(), "terman-go-cache")
	}
	modules := os.Getenv("GOMODCACHE")
	if modules == "" {
		modules = filepath.Join(os.TempDir(), "terman-go-mod-cache")
	}
	return []string{"GOCACHE=" + cache, "GOMODCACHE=" + modules}
}
