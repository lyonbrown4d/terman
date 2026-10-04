package client

import (
	"bufio"
	"context"
	"encoding/json"
	"net"
	"strings"
	"testing"
	"time"

	"github.com/gdamore/tcell/v3"
	"github.com/lyonbrown4d/terman/testkit/tuitest"
	"github.com/lyonbrown4d/terman/tmux/internal/protocol"
	"github.com/lyonbrown4d/terman/tmux/internal/store"
)

func TestAttachTUIEndToEnd(t *testing.T) {
	screen, server, result, cancel := startAttachHarness(t)
	defer cancel()
	defer server.Close()
	decoder := json.NewDecoder(bufio.NewReader(server))

	screen.Send(tcell.NewEventKey(tcell.KeyRune, "x", tcell.ModNone))
	request := decodeTmuxRequest(t, server, decoder)
	if request.Op != "input" || request.Data != "x" {
		t.Fatalf("keyboard request = %#v", request)
	}

	screen.SetSize(100, 30)
	screen.Send(tcell.NewEventResize(100, 30))
	request = decodeTmuxRequest(t, server, decoder)
	if request.Op != "resize-client" || request.Width != 100 || request.Height != 30 {
		t.Fatalf("resize request = %#v", request)
	}

	screen.Send(tcell.NewEventMouse(2, 1, tcell.Button1, tcell.ModNone))
	request = decodeTmuxRequest(t, server, decoder)
	if request.Op != "select-pane" || request.Pane == nil || *request.Pane != 0 {
		t.Fatalf("mouse selection request = %#v", request)
	}
	request = decodeTmuxRequest(t, server, decoder)
	if request.Op != "input" || request.Pane == nil || len(request.Data) == 0 {
		t.Fatalf("mouse input request = %#v", request)
	}

	if err := json.NewEncoder(server).Encode(protocol.Response{
		OK: true, Event: "frame", Frame: tmuxFrame("tmux-updated"),
	}); err != nil {
		t.Fatalf("send updated frame: %v", err)
	}
	waitTmuxFrame(t, screen, "tmux-updated")

	screen.Send(tcell.NewEventKey(tcell.KeyCtrlB, "b", tcell.ModCtrl))
	screen.Send(tcell.NewEventKey(tcell.KeyRune, "d", tcell.ModNone))
	request = decodeTmuxRequest(t, server, decoder)
	if request.Op != "detach" {
		t.Fatalf("detach request = %#v", request)
	}
	waitTmuxExit(t, screen, result)
}

func TestAttachContextCancellationRestoresTerminal(t *testing.T) {
	screen, server, result, cancel := startAttachHarness(t)
	defer server.Close()
	cancel()
	waitTmuxExit(t, screen, result)
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
		result <- attachWith(ctx, store.Record{Name: "e2e"}, attachDependencies{
			openStream: func(context.Context, store.Record) (*Stream, error) {
				return &Stream{
					Conn:    clientConn,
					Encoder: json.NewEncoder(clientConn),
					Decoder: json.NewDecoder(bufio.NewReader(clientConn)),
				}, nil
			},
			screen: func() (tcell.Screen, error) {
				return screen, nil
			},
		})
	}()

	decoder := json.NewDecoder(bufio.NewReader(serverConn))
	request := decodeTmuxRequest(t, serverConn, decoder)
	if request.Op != "attach" || request.ClientID == "" ||
		request.Width != 80 || request.Height != 24 {
		t.Fatalf("attach request = %#v", request)
	}
	if err := json.NewEncoder(serverConn).Encode(protocol.Response{
		ID: request.ID, OK: true, Event: "attached", Frame: tmuxFrame("tmux-ready"),
	}); err != nil {
		t.Fatalf("send attach response: %v", err)
	}
	waitTmuxFrame(t, screen, "tmux-ready")
	return screen, serverConn, result, cancel
}

func tmuxFrame(text string) *protocol.Frame {
	row := make([]protocol.Cell, 0, len(text))
	for _, character := range text {
		row = append(row, protocol.Cell{Content: string(character), Width: 1})
	}
	return &protocol.Frame{
		Session:    "e2e",
		Window:     0,
		ActivePane: 0,
		Status:     "0:shell",
		Panes: []protocol.PaneFrame{{
			Index:  0,
			Rect:   protocol.Rect{X: 0, Y: 0, W: 80, H: 23},
			Cells:  [][]protocol.Cell{row},
			Active: true,
		}},
	}
}

func decodeTmuxRequest(
	t *testing.T,
	conn net.Conn,
	decoder *json.Decoder,
) protocol.Request {
	t.Helper()
	_ = conn.SetReadDeadline(time.Now().Add(2 * time.Second))
	var request protocol.Request
	if err := decoder.Decode(&request); err != nil {
		t.Fatalf("decode tmux request: %v", err)
	}
	_ = conn.SetReadDeadline(time.Time{})
	return request
}

func waitTmuxFrame(t *testing.T, screen *tuitest.Screen, text string) {
	t.Helper()
	if frame, ok := screen.WaitFrame(2*time.Second, func(value string) bool {
		return strings.Contains(value, text)
	}); !ok {
		t.Fatalf("tmux frame does not contain %q:\n%s", text, frame)
	}
}

func waitTmuxExit(
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
		"paste disable":   screen.PasteDisabled(),
		"screen finalize": screen.Finalized(),
	} {
		select {
		case <-signal:
		case <-time.After(2 * time.Second):
			t.Fatalf("%s was not called", name)
		}
	}
}
