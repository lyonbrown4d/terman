package app

import (
	"fmt"
	"strconv"
	"strings"
)

type Args struct {
	Session, Command, Execute, Window string
	ExecuteArgs                       []string
	Cols, Rows                        int
	Detached, DetachExisting          bool
	List, JSON, Wipe, LoginShell      bool
	Resume, ResumeCreate, Multi       bool
	AttachName                        string
	Server, Help, Version             bool
	Endpoint                          string
}

func Parse(argv []string) (Args, error) {
	args := Args{Cols: 80, Rows: 24}
	values := normalize(argv[1:])
	for index := 0; index < len(values); index++ {
		value := values[index]
		next := func() (string, error) {
			index++
			if index >= len(values) {
				return "", fmt.Errorf("%s requires a value", value)
			}
			return values[index], nil
		}
		switch value {
		case "-S", "--session":
			item, err := next()
			if err != nil {
				return args, err
			}
			args.Session = item
		case "-d", "--detached":
			args.Detached = true
		case "-D", "--detach-existing":
			args.DetachExisting = true
		case "-r", "--resume":
			args.Resume = true
			if index+1 < len(values) && !strings.HasPrefix(values[index+1], "-") {
				index++
				args.AttachName = values[index]
			}
		case "-R", "--resume-or-create":
			args.ResumeCreate = true
			if index+1 < len(values) && !strings.HasPrefix(values[index+1], "-") {
				index++
				args.AttachName = values[index]
			}
		case "-x", "--multi-attach":
			args.Multi = true
			if index+1 < len(values) && !strings.HasPrefix(values[index+1], "-") {
				index++
				args.AttachName = values[index]
			}
		case "--list", "-ls", "-list":
			args.List = true
		case "--json":
			args.JSON = true
		case "--wipe", "-wipe":
			args.Wipe = true
		case "-X", "-Q", "--execute", "--query":
			item, err := next()
			if err != nil {
				return args, err
			}
			args.Execute = item
			args.ExecuteArgs = append(args.ExecuteArgs, values[index+1:]...)
			index = len(values)
		case "-p", "--window":
			item, err := next()
			if err != nil {
				return args, err
			}
			args.Window = item
		case "-c", "--command":
			item, err := next()
			if err != nil {
				return args, err
			}
			args.Command = item
		case "--cols":
			item, err := next()
			if err != nil {
				return args, err
			}
			args.Cols, err = strconv.Atoi(item)
			if err != nil {
				return args, err
			}
		case "--rows":
			item, err := next()
			if err != nil {
				return args, err
			}
			args.Rows, err = strconv.Atoi(item)
			if err != nil {
				return args, err
			}
		case "--login-shell":
			args.LoginShell = true
		case "--__screen-server":
			args.Server = true
		case "--__endpoint-name":
			item, err := next()
			if err != nil {
				return args, err
			}
			args.Endpoint = item
		case "-h", "--help":
			args.Help = true
		case "-v", "--version":
			args.Version = true
		case "--":
			args.Command = strings.Join(values[index+1:], " ")
			index = len(values)
		case "-m":
		default:
			if strings.HasPrefix(value, "-") {
				return args, fmt.Errorf("unknown option %s", value)
			}
			args.Command = strings.Join(values[index:], " ")
			index = len(values)
		}
	}
	if args.Detached && (args.Resume || args.ResumeCreate || args.Multi) {
		args.DetachExisting = true
		args.Detached = false
	}
	return args, nil
}

func normalize(values []string) []string {
	result := make([]string, 0, len(values)+2)
	for _, value := range values {
		switch {
		case value == "-dm":
			result = append(result, "-d")
		case value == "-dmS":
			result = append(result, "-d", "-S")
		case strings.HasPrefix(value, "-dmS") && len(value) > 4:
			result = append(result, "-d", "-S", value[4:])
		default:
			result = append(result, value)
		}
	}
	return result
}
