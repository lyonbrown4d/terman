package service

import (
	"context"
	"github.com/arcgolabs/dix"
	"testing"
)

func TestHandleLeavesOrdinaryArguments(t *testing.T) {
	handled, err := Handle(t.Context(), Config{}, []string{"new", "session"})
	if handled || err != nil {
		t.Fatalf("ordinary command: %v %v", handled, err)
	}
}

func TestHandleRejectsInvalidServiceCommands(t *testing.T) {
	for _, args := range [][]string{{"service"}, {"service", "invalid"}, {"service", "status", "extra"}} {
		t.Run(args[len(args)-1], func(t *testing.T) {
			handled, err := Handle(t.Context(), Config{Tool: "test-tool", ServerArgument: "--server"}, args)
			if !handled || err == nil {
				t.Fatalf("invalid command: %v %v", handled, err)
			}
		})
	}
}

func TestTryStartValidatesConfigAndCancellation(t *testing.T) {
	_, used, err := TryStart(t.Context(), Config{Tool: "../bad", ServerArgument: "--server"}, Launch{Name: "test"})
	if used || err == nil {
		t.Fatalf("invalid config: %v %v", used, err)
	}
	ctx, cancel := context.WithCancel(t.Context())
	cancel()
	_, used, err = TryStart(ctx, Config{Tool: "test-tool", ServerArgument: "--server"}, Launch{Name: "test"})
	if used || err == nil {
		t.Fatalf("cancelled request: %v %v", used, err)
	}
}

func TestModuleProvidesConfig(t *testing.T) {
	cfg := Config{Tool: "test-tool", ServerArgument: "--server"}
	rt, err := dix.New("service-test", dix.Modules(Module(cfg))).Start(t.Context())
	if err != nil {
		t.Fatal(err)
	}
	defer rt.Stop(t.Context())
	got, err := rt.Container().Resolve[Config]()
	if err != nil || got.Tool != cfg.Tool {
		t.Fatalf("config: %+v %v", got, err)
	}
}

func TestEndpointUsesToolAndUser(t *testing.T) {
	identity, err := UserIdentity()
	if err != nil || identity == "" {
		t.Fatalf("identity: %q %v", identity, err)
	}
	first, err := Endpoint(Config{Tool: "one", ServerArgument: "--server"})
	if err != nil {
		t.Fatal(err)
	}
	second, err := Endpoint(Config{Tool: "two", ServerArgument: "--server"})
	if err != nil || first == second {
		t.Fatalf("endpoints: %q %q %v", first, second, err)
	}
}
func TestServiceNameIsRestrictedToToolNamespace(t *testing.T) {
	for _, name := range []string{"unrelated", "test-tool-../bad", "test-tool-"} {
		t.Run(name, func(t *testing.T) {
			_, err := Endpoint(Config{Tool: "test-tool", ServiceName: name, ServerArgument: "--server"})
			if err == nil {
				t.Fatal("unsafe service name accepted")
			}
		})
	}
	t.Setenv("TERMAN_SERVICE_INSTANCE", "isolated")
	cfg := Config{Tool: "test-tool", ServerArgument: "--server"}
	name, err := effectiveName(cfg)
	if err != nil || name != "test-tool-isolated" {
		t.Fatalf("instance: %q %v", name, err)
	}
}
func TestServiceHelpIsHandledWithoutError(t *testing.T) {
	for _, args := range [][]string{{"service", "--help"}, {"service", "run", "--help"}, {"service", "install", "--help"}} {
		t.Run(args[1], func(t *testing.T) {
			handled, err := Handle(t.Context(), Config{Tool: "test-tool", ServerArgument: "--server"}, args)
			if !handled || err != nil {
				t.Fatalf("help: %v %v", handled, err)
			}
		})
	}
}
