package service

import (
	"context"
	"fmt"
	"os"
	"os/signal"
	"runtime"
	"syscall"

	native "github.com/kardianos/service"
)

type nativeConfig = native.Config

func nativeService(cfg Config, opts commandOptions, install bool, program native.Interface) (native.Service, error) {
	if err := validatePlatform(); err != nil {
		return nil, err
	}
	name, err := effectiveName(cfg)
	if err != nil {
		return nil, err
	}
	executable, err := os.Executable()
	if err != nil {
		return nil, err
	}
	config := &native.Config{Name: name, DisplayName: name, Description: "Per-user session broker for " + cfg.Tool,
		Executable: executable, Arguments: []string{"service", "run", "--name", name},
		Option: native.KeyValue{"UserService": true}}
	if install {
		if err = configurePlatform(config, opts.username, opts.passwordEnv); err != nil {
			return nil, err
		}
	}
	if program == nil {
		program = &inertProgram{}
	}
	return native.New(program, config)
}

type inertProgram struct{}

func (*inertProgram) Start(native.Service) error { return nil }
func (*inertProgram) Stop(native.Service) error  { return nil }

func runBroker(ctx context.Context, cfg Config) error {
	if err := restoreUserProfile(); err != nil {
		return err
	}
	program, err := newBroker(cfg)
	if err != nil {
		return err
	}
	if runtime.GOOS == "windows" && !native.Interactive() {
		svc, err := nativeService(cfg, commandOptions{}, false, program)
		if err != nil {
			_ = program.Stop(nil)
			return err
		}
		defer program.Stop(nil)
		return svc.Run()
	}
	signals, cancel := signal.NotifyContext(ctx, os.Interrupt, syscall.SIGTERM)
	defer cancel()
	if err = program.Start(nil); err != nil {
		_ = program.Stop(nil)
		return err
	}
	select {
	case <-signals.Done():
		return program.Stop(nil)
	case <-program.done:
		return program.stopErr
	}
}

func serviceError(key string, values map[string]string) error {
	return fmt.Errorf("%s", localized(key, values))
}
