package common

import (
	"os"
	"path/filepath"
	"runtime"
	"strings"
)

// ShellCommandArgs returns the switch used to execute a command string.
func ShellCommandArgs(shell string, loginShell bool) []string {
	name := strings.ToLower(filepath.Base(shell))
	switch {
	case strings.Contains(name, "cmd.exe"):
		return []string{"/C"}
	case strings.Contains(name, "powershell"), strings.Contains(name, "pwsh"):
		return []string{"-Command"}
	case strings.HasSuffix(name, "bash"), strings.HasSuffix(name, "bash.exe"),
		strings.HasSuffix(name, "sh"), strings.HasSuffix(name, "sh.exe"):
		if loginShell {
			return []string{"-lc"}
		}
		return []string{"-c"}
	default:
		return []string{"-c"}
	}
}

// DefaultShell selects the platform shell, honoring the conventional environment variable.
func DefaultShell() string {
	if runtime.GOOS == "windows" {
		if shell := firstNonEmptyEnv("COMSPEC", "ComSpec"); shell != "" {
			return shell
		}
		return "cmd.exe"
	}
	if shell := os.Getenv("SHELL"); shell != "" {
		return shell
	}
	return "/bin/sh"
}

func firstNonEmptyEnv(names ...string) string {
	for _, name := range names {
		if value := os.Getenv(name); value != "" {
			return value
		}
	}
	return ""
}
