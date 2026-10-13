package service

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net"
	"time"
)

const rpcTimeout = 5 * time.Second
const maxRequestBytes = 64 * 1024

type request struct {
	Operation string
	Launch    Launch
}
type response struct {
	Name     string
	Status   string
	Sessions int
	PID      int
	Error    string
}

// TryStart asks the current user's broker to start a session. An unavailable
// broker returns used=false; a contacted broker always returns used=true.
func TryStart(ctx context.Context, cfg Config, launch Launch) (pid int, used bool, err error) {
	if err = ctx.Err(); err != nil {
		return 0, false, err
	}
	if err = validateConfig(cfg); err != nil {
		return 0, false, err
	}
	reply, used, err := exchange(ctx, cfg, request{Operation: "start", Launch: launch})
	if err != nil {
		return 0, used, err
	}
	if used && reply.PID <= 0 {
		return 0, true, fmt.Errorf("broker returned an invalid session PID")
	}
	return reply.PID, used, nil
}

func exchange(ctx context.Context, cfg Config, req request) (response, bool, error) {
	address, err := Endpoint(cfg)
	if err != nil {
		return response{}, false, err
	}
	bounded, cancel := context.WithTimeout(ctx, rpcTimeout)
	defer cancel()
	conn, err := dialEndpoint(bounded, address)
	if err != nil {
		if ctx.Err() != nil {
			return response{}, false, ctx.Err()
		}
		if endpointUnavailable(err) {
			return response{}, false, nil
		}
		return response{}, false, fmt.Errorf("connect to service broker: %w", err)
	}
	defer conn.Close()
	deadline, _ := bounded.Deadline()
	if err = conn.SetDeadline(deadline); err != nil {
		return response{}, true, err
	}
	stopCancellation := context.AfterFunc(bounded, func() { _ = conn.Close() })
	defer stopCancellation()
	data, err := json.Marshal(req)
	if err != nil {
		return response{}, true, err
	}
	if len(data) > maxRequestBytes {
		return response{}, true, fmt.Errorf("service request exceeds %d bytes", maxRequestBytes)
	}
	if _, err = conn.Write(append(data, '\n')); err != nil {
		return response{}, true, requestError(ctx, err)
	}
	var reply response
	if err = json.NewDecoder(io.LimitReader(conn, maxRequestBytes)).Decode(&reply); err != nil {
		return response{}, true, requestError(ctx, err)
	}
	if reply.Error != "" {
		return reply, true, fmt.Errorf("service broker: %s", reply.Error)
	}
	return reply, true, nil
}

func requestError(ctx context.Context, err error) error {
	if ctx.Err() != nil {
		return ctx.Err()
	}
	if errors.Is(err, net.ErrClosed) {
		return fmt.Errorf("service request interrupted: %w", err)
	}
	return fmt.Errorf("service request: %w", err)
}

func waitStopped(ctx context.Context, cfg Config) error {
	bounded, cancel := context.WithTimeout(ctx, 20*time.Second)
	defer cancel()
	ticker := time.NewTicker(25 * time.Millisecond)
	defer ticker.Stop()
	for {
		_, used, err := exchange(bounded, cfg, request{Operation: "status"})
		if !used && err == nil {
			return nil
		}
		select {
		case <-bounded.Done():
			return fmt.Errorf("%s: %w", localized("service-stop-timeout", nil), bounded.Err())
		case <-ticker.C:
		}
	}
}
