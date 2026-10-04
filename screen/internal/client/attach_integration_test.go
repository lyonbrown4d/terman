package client

import (
	"context"
	"encoding/json"
	"net"
	"strings"
	"testing"
	"time"

	"github.com/gdamore/tcell/v3"
	"github.com/lyonbrown4d/terman/screen/internal/proto"
	"github.com/lyonbrown4d/terman/testkit/tuitest"
)

func TestAttachTUIEndToEnd(t *testing.T) {
	screen, server, result, cancel := startAttachHarness(t)
	defer cancel()
	defer server.Close()
	decoder := json.NewDecoder(server)

	screen.Send(tcell.NewEventKey(tcell.KeyRune, "x", tcell.ModNone))
	request := decodeScreenRequest(t, server, decoder)
	if request.Type != "input" || string(request.Data) != "x" {
		t.Fatalf("keyboard request = %#v", request)
	}

	screen.SetSize(100, 30)
	screen.Send(tcell.NewEventResize(100, 30))
	request = decodeScreenRequest(t, server, decoder)
	if request.Type != "resize" || request.Cols != 100 || request.Rows != 30 {
		t.Fatalf("resize request = %#v", request)
	}

	screen.Send(tcell.NewEventMouse(2, 1, tcell.Button1, tcell.ModNone))
	request = decodeScreenRequest(t, server, decoder)
	if request.Type != "input" || len(request.Data) == 0 {
		t.Fatalf("mouse request = %#v", request)
	}

	encoder := json.NewEncoder(server)
	if err := encoder.Encode(screenFrame("screen-updated")); err != nil {
		t.Fatalf("send updated frame: %v", err)
	}
	waitScreenFrame(t, screen, "screen-updated")

	screen.Send(tcell.NewEventKey(tcell.KeyCtrlA, "a", tcell.ModCtrl))
	screen.Send(tcell.NewEventKey(tcell.KeyRune, "d", tcell.ModNone))
	request = decodeScreenRequest(t, server, decoder)
	if request.Type != "detach" {
		t.Fatalf("detach request = %#v", request)
	}
	waitScreenExit(t, screen, result)
}

func TestAttachContextCancellationRestoresTerminal(t *testing.T) {
	screen, server, result, cancel := startAttachHarness(t)
	defer server.Close()
	decoder := json.NewDecoder(server)

	cancel()
	request := decodeScreenRequest(t, server, decoder)
	if request.Type != "detach" {
		t.Fatalf("cancel request = %#v", request)
	}
	waitScreenExit(t, screen, result)
}

func startAttachHarness(
	t *testing.T,
) (*tuitest.Screen, net.Conn, <-chan error, context.CancelFunc) {
	t.Helper()
	clientConn, serverConn := net.Pipe()
	screen := tuitest.NewScreen(80, 24)
	ctx, cancel := context.WithCancel(context.Background())
	result := make(chan error, 1)
	go func() {
		result <- attachWith(ctx, "test", "resume", false, attachDependencies{
			dial: func(context.Context, string) (net.Conn, error) {
				return clientConn, nil
			},
			screen: func() (tcell.Screen, error) {
				return screen, nil
			},
		})
	}()

	decoder := json.NewDecoder(serverConn)
	request := decodeScreenRequest(t, serverConn, decoder)
	if request.Type != "attach" || request.Mode != "resume" || request.ClientID == "" {
		t.Fatalf("attach request = %#v", request)
	}
	if err := json.NewEncoder(serverConn).Encode(screenFrame("screen-ready")); err != nil {
		t.Fatalf("send initial frame: %v", err)
	}
	waitScreenFrame(t, screen, "screen-ready")
	return screen, serverConn, result, cancel
}

func screenFrame(text string) proto.Response {
	return proto.Response{
		Type: "frame",
		Frame: &proto.Frame{
			Session: "e2e",
			Cols:    80,
			Rows:    24,
			Windows: []proto.Window{{Index: 0, Title: "shell", Active: true}},
			Regions: []proto.Region{{
				Index: 0, X: 0, Y: 0, Width: 80, Height: 23,
				Focused: true, Window: 0, Lines: []string{text},
			}},
		},
	}
}

func decodeScreenRequest(
	t *testing.T,
	conn net.Conn,
	decoder *json.Decoder,
) proto.Request {
	t.Helper()
	_ = conn.SetReadDeadline(time.Now().Add(2 * time.Second))
	var request proto.Request
	if err := decoder.Decode(&request); err != nil {
		t.Fatalf("decode screen request: %v", err)
	}
	_ = conn.SetReadDeadline(time.Time{})
	return request
}

func waitScreenFrame(t *testing.T, screen *tuitest.Screen, text string) {
	t.Helper()
	if frame, ok := screen.WaitFrame(2*time.Second, func(value string) bool {
		return strings.Contains(value, text)
	}); !ok {
		t.Fatalf("screen frame does not contain %q:\n%s", text, frame)
	}
}

func waitScreenExit(
	t *testing.T,
	screen *tuitest.Screen,
	result <-chan error,
) {
	t.Helper()
	select {
	case err := <-result:
		if err != nil {
			t.Fatalf("attach returned an error: %v", err)
		}
	case <-time.After(2 * time.Second):
		t.Fatal("attach did not exit")
	}
	for name, signal := range map[string]<-chan struct{}{
		"mouse disable":   screen.MouseDisabled(),
		"screen finalize": screen.Finalized(),
	} {
		select {
		case <-signal:
		case <-time.After(2 * time.Second):
			t.Fatalf("%s was not called", name)
		}
	}
}
