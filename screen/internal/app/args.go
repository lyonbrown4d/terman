package app

import "strings"

type Args struct {
	Session, Command, Execute, Window string
	ExecuteArgs                       []string
	Detached, DetachExisting          bool
	List, JSON, Wipe                  bool
	Resume, ResumeCreate, Multi       bool
	AttachName                        string
	Server, Version                   bool
	Endpoint                          string
}

func normalize(values []string) []string {
	result := make([]string, 0, len(values)+3)
	for index := 0; index < len(values); index++ {
		value := values[index]
		switch {
		case value == "-dm":
			result = append(result, "-d", "-m")
		case value == "-dmS":
			result = append(result, "-d", "-m", "-S")
		case strings.HasPrefix(value, "-dmS") && len(value) > 4:
			result = append(result, "-d", "-m", "-S", value[4:])
		case value == "-ls" || value == "-list":
			result = append(result, "--list")
		case value == "-wipe":
			result = append(result, "--wipe")
		case isExecuteFlag(value) && index+1 < len(values):
			result = append(result, value, values[index+1], "--")
			return append(result, values[index+2:]...)
		case strings.HasPrefix(value, "--execute=") || strings.HasPrefix(value, "--query="):
			result = append(result, value, "--")
			return append(result, values[index+1:]...)
		default:
			result = append(result, value)
		}
	}
	return result
}

func isExecuteFlag(value string) bool {
	return value == "-X" || value == "-Q" ||
		value == "--execute" || value == "--query"
}
