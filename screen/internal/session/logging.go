package session

import (
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"github.com/lyonbrown4d/terman/screen/internal/proto"
)

func (o *Owner) logCommand(command string, args []string, target string) proto.Response {
	switch command {
	case "logfile":
		if len(args) == 0 {
			return accepted(o.logfile)
		}
		if len(args) == 2 && args[0] == "flush" {
			if _, err := strconv.Atoi(args[1]); err != nil {
				return rejected(fmt.Errorf("invalid logfile flush interval: %w", err))
			}
			return accepted("")
		}
		if len(args) != 1 {
			return rejected(fmt.Errorf("logfile requires one path"))
		}
		o.logfile = args[0]
	case "deflog":
		value, err := parseSwitch(args)
		if err != nil {
			return rejected(err)
		}
		o.deflog = value
	case "log":
		index := o.targetWindow(target)
		if index < 0 {
			return rejected(fmt.Errorf("window not found"))
		}
		config := o.logs[o.windows[index].ID]
		if len(args) == 0 {
			config.enabled = !config.enabled
		} else {
			value, err := parseSwitch(args)
			if err != nil {
				return rejected(err)
			}
			config.enabled = value
		}
		config.last = time.Time{}
		o.logs[o.windows[index].ID] = config
	case "logtstamp":
		if err := o.configureLogTimestamp(args); err != nil {
			return rejected(err)
		}
	}
	return accepted("")
}

func (o *Owner) configureLogTimestamp(args []string) error {
	if len(args) == 0 {
		o.logTimestamp = !o.logTimestamp
		return nil
	}
	if len(args) == 1 {
		value, err := parseSwitch(args)
		if err != nil {
			return err
		}
		o.logTimestamp = value
		return nil
	}
	if len(args) == 2 && args[0] == "after" {
		seconds, err := strconv.Atoi(args[1])
		if err != nil || seconds < 0 {
			return fmt.Errorf("logtstamp after requires non-negative seconds")
		}
		o.logAfter = time.Duration(seconds) * time.Second
		o.logTimestamp = true
		return nil
	}
	if len(args) >= 2 && args[0] == "string" {
		o.logStamp = strings.Join(args[1:], " ")
		o.logTimestamp = true
		return nil
	}
	return fmt.Errorf("logtstamp requires on, off, after seconds, or string text")
}

func (o *Owner) writeLog(index int, data []byte) {
	window := o.windows[index]
	config := o.logs[window.ID]
	if !config.enabled {
		return
	}
	path := o.logPath(index)
	file, err := os.OpenFile(path, os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0o600)
	if err != nil {
		o.lastMessage = fmt.Sprintf("log: %v", err)
		return
	}
	now := time.Now()
	if o.logTimestamp && !config.last.IsZero() && now.Sub(config.last) >= o.logAfter {
		if _, err := file.WriteString(formatStamp(o.logStamp, now)); err != nil {
			o.lastMessage = fmt.Sprintf("log timestamp: %v", err)
		}
	}
	if _, err := file.Write(data); err != nil {
		o.lastMessage = fmt.Sprintf("log write: %v", err)
	}
	if err := file.Close(); err != nil {
		o.lastMessage = fmt.Sprintf("log close: %v", err)
	}
	config.last = now
	o.logs[window.ID] = config
}

func (o *Owner) logPath(index int) string {
	title := sanitizeFilename(o.windows[index].Title)
	value := strings.NewReplacer(
		"%n", strconv.Itoa(index),
		"%t", title,
		"%S", sanitizeFilename(o.config.Name),
	).Replace(o.logfile)
	if filepath.IsAbs(value) {
		return filepath.Clean(value)
	}
	return filepath.Join(o.cwd, filepath.Clean(value))
}

func formatStamp(format string, value time.Time) string {
	result := strings.NewReplacer(
		"%Y", value.Format("2006"),
		"%m", value.Format("01"),
		"%d", value.Format("02"),
		"%H", value.Format("15"),
		"%M", value.Format("04"),
		"%S", value.Format("05"),
		"\\n", "\n",
	).Replace(format)
	if !strings.HasSuffix(result, "\n") {
		result += "\n"
	}
	return result
}

func sanitizeFilename(value string) string {
	return strings.Map(func(char rune) rune {
		switch char {
		case '<', '>', ':', '"', '/', '\\', '|', '?', '*':
			return '_'
		}
		if char < 32 {
			return '_'
		}
		return char
	}, value)
}
