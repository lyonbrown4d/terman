package session

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/lyonbrown4d/terman/screen/internal/proto"
)

func (o *Owner) captureCommand(command string, args []string, target string) proto.Response {
	switch command {
	case "hardcopydir":
		if len(args) != 1 {
			return rejected(fmt.Errorf("hardcopydir requires a directory"))
		}
		path := o.resolvePath(args[0])
		info, err := os.Stat(path)
		if err != nil || !info.IsDir() {
			return rejected(fmt.Errorf("hardcopy directory %q is not a directory", path))
		}
		o.hardcopyDir = path
		return accepted(path)
	case "hardcopy_append":
		value, err := parseSwitch(args)
		if err != nil {
			return rejected(err)
		}
		o.hardcopyAppend = value
	case "hardcopy":
		message, err := o.hardcopy(args, target)
		if err != nil {
			return rejected(err)
		}
		return accepted(message)
	}
	return accepted("")
}

func (o *Owner) hardcopy(args []string, target string) (string, error) {
	history := false
	path := ""
	for _, arg := range args {
		if arg == "-h" {
			history = true
			continue
		}
		if path != "" {
			return "", fmt.Errorf("hardcopy accepts one output path")
		}
		path = arg
	}
	index := o.targetWindow(target)
	if index < 0 {
		return "", fmt.Errorf("window not found")
	}
	if path == "" {
		directory := o.hardcopyDir
		if directory == "" {
			directory = o.cwd
		}
		path = filepath.Join(directory, fmt.Sprintf("hardcopy.%d", index))
	} else {
		path = o.resolvePath(path)
	}
	lines := o.windows[index].Lines()
	if history {
		lines = o.windows[index].HistoryLines()
	}
	data := []byte(strings.Join(lines, "\n") + "\n")
	if err := writePrivate(path, data, o.hardcopyAppend); err != nil {
		return "", err
	}
	return fmt.Sprintf("wrote %d bytes to %s", len(data), path), nil
}

func (o *Owner) resolvePath(path string) string {
	if filepath.IsAbs(path) {
		return filepath.Clean(path)
	}
	return filepath.Join(o.cwd, filepath.Clean(path))
}
