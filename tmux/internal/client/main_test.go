package client

import (
	"testing"

	"go.uber.org/goleak"
)

func TestMain(m *testing.M) {
	// The service module initializes ants' process-wide pool at import time.
	goleak.VerifyTestMain(m, goleak.IgnoreCurrent())
}
