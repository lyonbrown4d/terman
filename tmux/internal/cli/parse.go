package cli

import (
	"strconv"
	"strings"
)

func firstCommand(args []string) string {
	for _, arg := range args {
		if arg == "-d" || arg == "--detached" {
			continue
		}
		if !strings.HasPrefix(arg, "-") {
			return arg
		}
	}
	return ""
}

func option(args []string, name, fallback string) string {
	return optionAny(args, []string{name}, fallback)
}

func optionAny(args []string, names []string, fallback string) string {
	for i, arg := range args {
		for _, name := range names {
			if arg == name && i+1 < len(args) {
				return args[i+1]
			}
			if strings.HasPrefix(arg, name+"=") {
				return strings.TrimPrefix(arg, name+"=")
			}
			if len(name) == 2 && strings.HasPrefix(arg, name) && len(arg) > 2 {
				return arg[2:]
			}
		}
	}
	return fallback
}

func has(args []string, values ...string) bool {
	for _, arg := range args {
		for _, value := range values {
			if arg == value {
				return true
			}
		}
	}
	return false
}

func parseTarget(target string) (string, *int, *int) {
	if target == "" {
		return "", nil, nil
	}
	session := target
	var window, pane *int
	if colon := strings.IndexByte(target, ':'); colon >= 0 {
		session = target[:colon]
		selector := target[colon+1:]
		if dot := strings.LastIndexByte(selector, '.'); dot >= 0 {
			pane = intPointer(selector[dot+1:])
			selector = selector[:dot]
		}
		if selector != "" && selector != "." {
			window = intPointer(selector)
		}
	}
	return session, window, pane
}

func intPointer(value string) *int {
	number, err := strconv.Atoi(value)
	if err != nil || number < 0 {
		return nil
	}
	return &number
}

func windowArg(args []string) *int {
	target := optionAny(args, []string{"-t", "--target-session"}, "")
	_, window, _ := parseTarget(target)
	return window
}

func paneArg(args []string) *int {
	target := optionAny(args, []string{"-t", "--target-session"}, "")
	_, _, pane := parseTarget(target)
	return pane
}

func paneFrom(target string) *int {
	_, _, pane := parseTarget(target)
	if pane != nil {
		return pane
	}
	return intPointer(target)
}

func directionArg(args []string) string {
	switch {
	case has(args, "-L"):
		return "left"
	case has(args, "-R"):
		return "right"
	case has(args, "-U"):
		return "up"
	case has(args, "-D"):
		return "down"
	}
	return ""
}

func firstPositional(args []string, command string) string {
	values := positionalValues(args, command)
	if len(values) == 0 {
		return ""
	}
	return values[0]
}

func positionalValues(args []string, command string) []string {
	return strings.Fields(payloadAfter(args, []string{command}, map[string]bool{
		"-t": true, "--target-session": true, "-b": true, "--buffer-name": true,
		"-x": true, "--width": true, "-y": true, "--height": true,
		"-s": true, "--source-pane": true,
	}))
}

func payloadAfter(args, commands []string, consumes map[string]bool) string {
	found := false
	var output []string
	for i := 0; i < len(args); i++ {
		arg := args[i]
		if !found {
			for _, command := range commands {
				if arg == command {
					found = true
					break
				}
			}
			continue
		}
		if consumes[arg] {
			i++
			continue
		}
		skip := false
		for option := range consumes {
			if strings.HasPrefix(option, "--") && strings.HasPrefix(arg, option+"=") {
				skip = true
			}
			if len(option) == 2 && strings.HasPrefix(arg, option) && len(arg) > 2 {
				skip = true
			}
		}
		if skip || strings.HasPrefix(arg, "-") {
			continue
		}
		output = append(output, arg)
	}
	return strings.Join(output, " ")
}
