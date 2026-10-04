package app

import (
	"context"
	"fmt"
	"os"
	"strings"

	"github.com/lyonbrown4d/terman/common"
	"github.com/lyonbrown4d/terman/screen/internal/session"
	"github.com/spf13/cobra"
	"github.com/spf13/pflag"
)

func Run(ctx context.Context, argv []string) error {
	args := Args{}
	defaults := defaultPersistentConfig()
	configFlags := pflag.NewFlagSet("screen-config", pflag.ContinueOnError)
	if err := bindConfigFlags(configFlags, defaults); err != nil {
		return fmt.Errorf("bind screen config flags: %w", err)
	}
	root, err := newRootCommand(&args, configFlags, defaults)
	if err != nil {
		return err
	}
	root.SetArgs(normalize(argv[1:]))
	root.SetOut(os.Stdout)
	root.SetErr(os.Stderr)
	return root.ExecuteContext(ctx)
}

func newRootCommand(
	args *Args,
	configFlags *pflag.FlagSet,
	defaults PersistentConfig,
) (*cobra.Command, error) {
	root := &cobra.Command{
		Use:           "terman-screen [flags] [command]",
		Short:         "native cross-platform terminal sessions",
		Long:          common.LocalizedMessage("builtin-screen-cli-about", nil),
		SilenceErrors: true,
		SilenceUsage:  true,
		Args:          cobra.ArbitraryArgs,
		RunE: func(command *cobra.Command, positional []string) error {
			if args.Version {
				fmt.Fprintln(command.OutOrStdout(), "terman-screen "+session.Version)
				return nil
			}
			if err := finalizeArgs(args, positional); err != nil {
				return err
			}
			config, err := loadPersistentConfig(command.Context(), configFlags, defaults)
			if err != nil {
				return fmt.Errorf("load screen config: %w", err)
			}
			return execute(command.Context(), *args, config)
		},
	}
	root.CompletionOptions.DisableDefaultCmd = true
	flags := root.Flags()
	flags.StringVarP(&args.Session, "session", "S", "", "session name")
	flags.BoolVarP(&args.Detached, "detached", "d", false, "start detached")
	flags.BoolVarP(&args.DetachExisting, "detach-existing", "D", false, "detach existing clients")
	flags.BoolVarP(&args.Resume, "resume", "r", false, "resume a session")
	flags.BoolVarP(&args.ResumeCreate, "resume-or-create", "R", false, "resume or create a session")
	flags.BoolVarP(&args.Multi, "multi-attach", "x", false, "attach without detaching clients")
	flags.BoolVar(&args.List, "list", false, "list sessions")
	flags.BoolVar(&args.JSON, "json", false, "emit JSON")
	flags.BoolVar(&args.Wipe, "wipe", false, "remove dead session records")
	flags.StringVarP(&args.Execute, "execute", "X", "", "execute a control command")
	flags.StringVarP(&args.Execute, "query", "Q", "", "query a control command")
	flags.StringVarP(&args.Window, "window", "p", "", "target window")
	flags.StringVarP(&args.Command, "command", "c", "", "shell command")
	flags.BoolVarP(&args.Version, "version", "v", false, "print version")
	flags.Bool("m", false, "force creation of a new session")
	flags.BoolVar(&args.Server, "__screen-server", false, "")
	flags.StringVar(&args.Endpoint, "__endpoint-name", "", "")
	if err := flags.MarkHidden("__screen-server"); err != nil {
		return nil, err
	}
	if err := flags.MarkHidden("__endpoint-name"); err != nil {
		return nil, err
	}
	flags.AddFlagSet(configFlags)
	root.MarkFlagsMutuallyExclusive("resume", "resume-or-create", "multi-attach")
	return root, nil
}

func finalizeArgs(args *Args, positional []string) error {
	if args.Execute != "" {
		args.ExecuteArgs = append([]string(nil), positional...)
		return normalizeModes(args)
	}
	isAttach := args.Resume || args.ResumeCreate || args.Multi
	if isAttach {
		if len(positional) > 1 {
			return fmt.Errorf("attach accepts at most one session name")
		}
		if len(positional) == 1 {
			args.AttachName = positional[0]
		}
		return normalizeModes(args)
	}
	if args.Command != "" && len(positional) > 0 {
		return fmt.Errorf("command specified by both --command and positional arguments")
	}
	if len(positional) > 0 {
		args.Command = strings.Join(positional, " ")
	}
	return normalizeModes(args)
}

func normalizeModes(args *Args) error {
	if args.Detached && (args.Resume || args.ResumeCreate || args.Multi) {
		args.DetachExisting = true
		args.Detached = false
	}
	return nil
}
