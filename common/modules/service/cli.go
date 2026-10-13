package service

import (
	"context"
	"errors"
	"flag"
	"fmt"
	"io"
	"os"
	"runtime"

	native "github.com/kardianos/service"
	common "github.com/lyonbrown4d/terman/common"
)

type commandOptions struct {
	name        string
	username    string
	passwordEnv string
	json        bool
}

// Handle consumes service subcommands from arguments without the executable.
func Handle(ctx context.Context, cfg Config, args []string) (bool, error) {
	if len(args) == 0 || args[0] != "service" {
		return false, nil
	}
	if err := ctx.Err(); err != nil {
		return true, err
	}
	if len(args) < 2 {
		return true, fmt.Errorf("%s", localized("service-usage", map[string]string{"tool": cfg.Tool}))
	}
	action := args[1]
	if action == "--help" || action == "-h" || action == "help" {
		return true, printHelp(cfg, "")
	}
	switch action {
	case "install", "uninstall", "start", "stop", "restart", "status", "run":
	default:
		return true, fmt.Errorf("%s", localized("service-unknown-action", map[string]string{"action": action}))
	}
	opts, err := parseOptions(action, args[2:])
	if errors.Is(err, flag.ErrHelp) {
		return true, printHelp(cfg, action)
	}
	if err != nil {
		return true, err
	}
	if opts.name != "" {
		cfg.ServiceName = opts.name
	}
	if err = validateConfig(cfg); err != nil {
		return true, err
	}
	if _, err = UserIdentity(); err != nil {
		return true, err
	}
	if action == "run" {
		return true, runBroker(ctx, cfg)
	}
	// SCM controls must reach its dispatcher so Windows reports a clean stop.
	if action == "stop" && runtime.GOOS == "windows" {
		svc, svcErr := nativeService(cfg, opts, false, nil)
		if svcErr == nil {
			status, statusErr := svc.Status()
			if statusErr == nil && status == native.StatusRunning {
				return true, svc.Stop()
			}
		}
	}
	if action == "status" || action == "stop" {
		reply, used, rpcErr := exchange(ctx, cfg, request{Operation: action})
		if used {
			if rpcErr != nil {
				return true, rpcErr
			}
			if action == "stop" {
				if err = waitStopped(ctx, cfg); err != nil {
					return true, err
				}
				return true, printAction(cfg, action)
			}
			return true, printStatus(reply, opts.json)
		}
		if rpcErr != nil {
			return true, rpcErr
		}
	}
	svc, err := nativeService(cfg, opts, action == "install", nil)
	if err != nil {
		return true, err
	}
	if action == "status" {
		return true, nativeStatus(cfg, svc, opts.json)
	}
	if action == "uninstall" {
		status, statusErr := svc.Status()
		if statusErr == nil && status == native.StatusRunning {
			if err = svc.Stop(); err != nil {
				return true, fmt.Errorf("stop service before uninstall: %w", err)
			}
		}
	}
	if err = native.Control(svc, action); err != nil {
		return true, err
	}
	if action == "install" {
		if err = grantServiceControl(cfg); err != nil {
			return true, err
		}
	}
	return true, printAction(cfg, action)
}

func parseOptions(action string, args []string) (commandOptions, error) {
	var opts commandOptions
	flags := flag.NewFlagSet("service "+action, flag.ContinueOnError)
	flags.SetOutput(io.Discard)
	flags.StringVar(&opts.name, "name", "", localized("service-flag-name", nil))
	if action == "status" {
		flags.BoolVar(&opts.json, "json", false, localized("service-flag-json", nil))
	}
	if action == "install" {
		flags.StringVar(&opts.username, "username", "", localized("service-flag-username", nil))
		flags.StringVar(&opts.passwordEnv, "password-env", "", localized("service-flag-password-env", nil))
	}
	if err := flags.Parse(args); err != nil {
		return opts, err
	}
	if flags.NArg() != 0 {
		return opts, fmt.Errorf("unexpected service arguments")
	}
	return opts, nil
}

func localized(key string, values map[string]string) string {
	return common.LocalizedMessage(key, values)
}

func printAction(cfg Config, action string) error {
	name, err := effectiveName(cfg)
	if err != nil {
		return err
	}
	_, err = fmt.Fprintln(os.Stdout, localized("service-action-done", map[string]string{"name": name, "action": action}))
	return err
}
