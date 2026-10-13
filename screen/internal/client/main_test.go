package client

import (
	"testing"

	"go.uber.org/goleak"
)

func TestMain(m *testing.M) {
	// The service module initializes ants' process-wide pool at import time.
	// go-winio starts one process-wide IO completion goroutine on first use.
	goleak.VerifyTestMain(m, goleak.IgnoreCurrent(),
		goleak.IgnoreAnyFunction("github.com/Microsoft/go-winio.ioCompletionProcessor"))
}
