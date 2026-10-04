package app

import (
	"github.com/lyonbrown4d/terman/testkit/tuitest"
)

type testScreen struct {
	*tuitest.Screen
}

func newTestScreen(width, height int) *testScreen {
	return &testScreen{Screen: tuitest.NewScreen(width, height)}
}

func testFrame(screen *testScreen) string {
	return screen.Frame()
}
