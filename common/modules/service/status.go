package service

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"

	native "github.com/kardianos/service"
)

func (b *broker) status() response {
	name, _ := effectiveName(b.cfg)
	b.mu.Lock()
	defer b.mu.Unlock()
	status := "running"
	if b.stopping {
		status = "stopping"
	}
	return response{Name: name, Status: status, PID: os.Getpid(), Sessions: len(b.sessions)}
}

func printStatus(reply response, jsonOutput bool) error {
	if jsonOutput {
		return json.NewEncoder(os.Stdout).Encode(struct {
			Name     string `json:"name"`
			Status   string `json:"status"`
			PID      int    `json:"pid"`
			Sessions int    `json:"sessions"`
		}{reply.Name, reply.Status, reply.PID, reply.Sessions})
	}
	_, err := fmt.Fprintln(os.Stdout, localized("service-status", map[string]string{"name": reply.Name, "status": localized("service-status-"+reply.Status, nil)}))
	return err
}

func nativeStatus(cfg Config, svc native.Service, jsonOutput bool) error {
	name, err := effectiveName(cfg)
	if err != nil {
		return err
	}
	status, err := svc.Status()
	if errors.Is(err, native.ErrNotInstalled) {
		return printStatus(response{Name: name, Status: "not-installed"}, jsonOutput)
	}
	if err != nil {
		return err
	}
	state := "stopped"
	if status == native.StatusRunning {
		state = "running"
	}
	if status == native.StatusUnknown {
		state = "unknown"
	}
	return printStatus(response{Name: name, Status: state}, jsonOutput)
}
