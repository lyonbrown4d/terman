package app

import "fmt"

type ProcessSignal int

const (
	SignalTerm ProcessSignal = iota
	SignalKill
	SignalInterrupt
	SignalHangup
	SignalStop
	SignalContinue
)

func (s ProcessSignal) String() string {
	names := [...]string{"TERM", "KILL", "INT", "HUP", "STOP", "CONT"}
	if int(s) < 0 || int(s) >= len(names) {
		return fmt.Sprintf("SIGNAL(%d)", s)
	}
	return names[s]
}
