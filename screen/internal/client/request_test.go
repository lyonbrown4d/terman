package client

import (
	"context"
	"fmt"
	"os"
	"testing"
	"time"

	"github.com/lyonbrown4d/terman/screen/internal/proto"
	"github.com/lyonbrown4d/terman/screen/internal/transport"
)

func TestRequestCancellationInterruptsResponseRead(t *testing.T) {
	endpoint, err := transport.Endpoint(fmt.Sprintf("request-cancel-%d", os.Getpid()))
	if err != nil {
		t.Fatal(err)
	}
	listener, err := transport.Listen(endpoint)
	if err != nil {
		t.Fatal(err)
	}
	defer listener.Close()
	accepted := make(chan struct{})
	go func() {
		conn, err := listener.Accept()
		if err != nil {
			return
		}
		defer conn.Close()
		close(accepted)
		// Bound the test even when the client fails to observe cancellation.
		select {
		case <-t.Context().Done():
		case <-time.After(300 * time.Millisecond):
		}
	}()
	ctx, cancel := context.WithCancel(t.Context())
	defer cancel()
	result := make(chan error, 1)
	go func() {
		_, err := Request(ctx, endpoint, proto.Request{Type: "ping"})
		result <- err
	}()
	select {
	case <-accepted:
	case <-time.After(time.Second):
		t.Fatal("request connection was not accepted")
	}
	cancel()
	select {
	case err := <-result:
		if err == nil {
			t.Fatal("canceled request succeeded")
		}
	case <-time.After(100 * time.Millisecond):
		t.Fatal("response read ignored context cancellation")
	}
}
